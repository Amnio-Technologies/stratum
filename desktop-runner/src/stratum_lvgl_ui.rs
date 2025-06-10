use crate::{
    flush_area_collector::FlushAreaCollector, fps_tracker::FpsTracker,
    lvgl_backend::DesktopLvglBackend,
};
use egui::{ColorImage, Context, TextureFilter, TextureHandle, TextureOptions};
use std::{
    pin::Pin,
    sync::mpsc::{channel, Receiver},
    sync::{Arc, LazyLock, Mutex},
    thread,
    time::{Duration, Instant},
};
use stratum_ui_common::{lvgl_backend::LvglBackend, stratum_ui_ffi};

pub static RENDER_LOCK: LazyLock<Arc<Mutex<()>>> = LazyLock::new(|| Arc::new(Mutex::new(())));

/// Texture sampling options (nearest‐neighbor)
fn default_tex_opts() -> TextureOptions {
    TextureOptions {
        minification: TextureFilter::Nearest,
        magnification: TextureFilter::Nearest,
        ..Default::default()
    }
}

/// Convert an RGB565 framebuffer into an egui::ColorImage (RGBA8)
fn rgb565_to_color_image(frame_buffer: &[u16], width: usize, height: usize) -> ColorImage {
    let mut rgba = Vec::with_capacity(width * height * 4);
    rgba.resize(width * height * 4, 0);

    for (i, &px) in frame_buffer.iter().enumerate() {
        let r = ((px >> 11) & 0x1F) << 3;
        let g = ((px >> 5) & 0x3F) << 2;
        let b = (px & 0x1F) << 3;
        let base = i * 4;
        rgba[base + 0] = r as u8;
        rgba[base + 1] = g as u8;
        rgba[base + 2] = b as u8;
        rgba[base + 3] = 0xFF;
    }

    ColorImage::from_rgba_unmultiplied([width, height], &rgba)
}

pub struct StratumLvglUI {
    /// Shared, pinned backend so its framebuffer pointer never moves
    backend: Arc<Mutex<Pin<Box<DesktopLvglBackend>>>>,
    /// Incoming frames from the LVGL thread
    frame_rx: Receiver<ColorImage>,
    /// The egui texture handle we update
    texture: Option<TextureHandle>,
    /// How often we actually produce a frame (None = vsync max)
    render_interval: Arc<Mutex<Duration>>,
    fps_tracker: Arc<Mutex<FpsTracker>>,
    pub flush_collector: Arc<FlushAreaCollector>,
}

impl StratumLvglUI {
    /// Spawn the LVGL logic + render thread, drive LVGL at high tick rate,
    /// snapshot only at `fps_limit`, and wake egui via `ctx.request_repaint()`.
    pub fn new(ctx: &Context, fps_limit: Option<u32>, repaint_flash_enabled: bool) -> Self {
        // initial interval (zero = no cap / vsync)
        let initial = fps_limit
            .map(|hz| Duration::from_secs_f64(1.0 / hz as f64))
            .unwrap_or_default();
        let render_interval = Arc::new(Mutex::new(initial));
        let fps_tracker = Arc::new(Mutex::new(FpsTracker::new()));

        // Shared, pinned backend
        let backend = Arc::new(Mutex::new(Box::pin(DesktopLvglBackend::new())));

        let flush_collector = FlushAreaCollector::new(repaint_flash_enabled);

        flush_collector.scope(|| {
            // initial UI setup
            let mut be = backend.lock().unwrap();
            be.as_mut().get_mut().setup_ui();
        });

        let (tx, rx) = channel();
        let ctx_clone = ctx.clone();
        let interval_handle = Arc::clone(&render_interval);
        let backend_handle = Arc::clone(&backend);
        let fps_tracker_handle = Arc::clone(&fps_tracker);
        let render_lock = Arc::clone(&RENDER_LOCK);
        let flush_collector_handle = flush_collector.clone();

        thread::spawn(move || {
            let mut last_frame = Instant::now();

            loop {
                let now = Instant::now();

                // This block holds both RENDER_LOCK and the backend mutex
                // only long enough to tick LVGL and grab the raw framebuffer.
                let maybe_fb: Option<Vec<u16>> = {
                    //  Lock render
                    let _render_guard = render_lock.lock().unwrap();
                    // Lock backend
                    let mut be = backend_handle.lock().unwrap();

                    //  LVGL tick/update
                    flush_collector_handle.scope(|| {
                        be.as_mut().get_mut().update_ui();
                    });

                    if !flush_collector_handle.active_events().is_empty() {
                        ctx_clone.request_repaint();
                    }

                    // 4. Decide if it's time to snapshot
                    let interval = *interval_handle.lock().unwrap();
                    if interval.is_zero() || now.duration_since(last_frame) >= interval {
                        last_frame = now;
                        // 5. Copy framebuffer under lock
                        Some(be.with_framebuffer(|fb| fb.to_vec()))
                    } else {
                        None
                    }
                };

                // If we grabbed a new frame, convert & send **after** unlocking
                if let Some(fb) = maybe_fb {
                    // Convert RGB565 → ColorImage
                    let (w, h) = unsafe {
                        (
                            stratum_ui_ffi::get_lvgl_display_width() as usize,
                            stratum_ui_ffi::get_lvgl_display_height() as usize,
                        )
                    };
                    let img = rgb565_to_color_image(&fb, w, h);

                    // Track & dispatch
                    fps_tracker_handle.lock().unwrap().tick();
                    let _ = tx.send(img);
                    ctx_clone.request_repaint();
                }

                thread::sleep(Duration::from_millis(1));
            }
        });

        Self {
            backend,
            frame_rx: rx,
            texture: None,
            render_interval,
            fps_tracker,
            flush_collector,
        }
    }

    /// Hot-reload the LVGL UI definition at runtime.
    pub fn reload_ui(&self) {
        self.flush_collector.clone().bind();
        let mut be = self.backend.lock().unwrap();
        be.as_mut().get_mut().setup_ui();
    }

    /// Change the user‐selected FPS cap at runtime.
    /// `None` means unlimited (vsync).
    pub fn set_fps_limit(&self, fps_limit: Option<u32>) {
        let new_interval = fps_limit
            .map(|hz| Duration::from_secs_f64(1.0 / hz as f64))
            .unwrap_or_default();
        *self.render_interval.lock().unwrap() = new_interval;
    }

    pub fn current_fps(&self) -> f64 {
        self.fps_tracker.lock().unwrap().fps()
    }

    /// Call from `eframe::App::update()`. Returns the latest texture to draw.
    pub fn update(&mut self, ctx: &Context) -> Option<TextureHandle> {
        // Drain any pending frames; keep only the latest
        let mut latest = None;
        while let Ok(img) = self.frame_rx.try_recv() {
            latest = Some(img);
        }

        if let Some(img) = latest {
            let opts = default_tex_opts();
            if let Some(tex) = &mut self.texture {
                tex.set(img, opts);
            } else {
                self.texture = Some(ctx.load_texture("lvgl_fb", img, opts));
            }
        }

        self.texture.clone()
    }
}
