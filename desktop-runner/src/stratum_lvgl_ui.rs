use std::pin::Pin;

use egui::{ColorImage, TextureHandle, TextureOptions};
use stratum_ui_common::{lvgl_backend::LvglBackend, stratum_ui_ffi};

use crate::lvgl_backend::DesktopLvglBackend;

pub struct StratumLvglUI {
    backend: Pin<Box<DesktopLvglBackend>>,
    renderer: LvglRenderer,
}

impl StratumLvglUI {
    pub fn new() -> Self {
        let mut backend = Box::pin(DesktopLvglBackend::new());
        backend.as_mut().get_mut().setup_ui();

        Self {
            backend,
            renderer: LvglRenderer::new(),
        }
    }

    pub fn reload_ui(&mut self) {
        self.backend.as_mut().get_mut().setup_ui();
    }

    pub fn update(&mut self, ctx: &egui::Context) -> Option<&TextureHandle> {
        self.backend.as_mut().get_mut().update_ui();

        self.backend
            .with_framebuffer(|fb| self.renderer.render_lvgl_framebuffer(fb, ctx));

        self.renderer.get_texture()
    }
}

struct LvglRenderer {
    texture: Option<TextureHandle>,
}

impl LvglRenderer {
    fn new() -> Self {
        LvglRenderer { texture: None }
    }

    /// Converts LVGL's RGB565 framebuffer to RGBA and uploads to GPU
    fn render_lvgl_framebuffer(&mut self, frame_buffer: &[u16], egui_ctx: &egui::Context) {
        // 1) Convert to RGBA_u8
        let (width, height) = unsafe {
            (
                stratum_ui_common::stratum_ui_ffi::get_lvgl_display_width() as usize,
                stratum_ui_common::stratum_ui_ffi::get_lvgl_display_height() as usize,
            )
        };
        let mut rgba_data = Vec::with_capacity(width * height * 4);
        rgba_data.resize(width * height * 4, 0);
        for (i, &pixel) in frame_buffer.iter().enumerate() {
            let r = ((pixel >> 11) & 0x1F) << 3;
            let g = ((pixel >> 5) & 0x3F) << 2;
            let b = (pixel & 0x1F) << 3;
            rgba_data[i * 4] = r as u8;
            rgba_data[i * 4 + 1] = g as u8;
            rgba_data[i * 4 + 2] = b as u8;
            rgba_data[i * 4 + 3] = 255 as u8;
        }
        let img = egui::ColorImage::from_rgba_unmultiplied([width, height], &rgba_data);

        // 2) If we already have a texture handle, just call .set() to update it
        if let Some(tex) = &mut self.texture {
            tex.set(img, egui::TextureOptions::default());
        } else {
            // First time only: allocate it
            self.texture = Some(egui_ctx.load_texture(
                "lvgl_fb", // the same id, persistent
                img,
                egui::TextureOptions::default(),
            ));
        }
    }

    fn get_texture(&self) -> Option<&TextureHandle> {
        self.texture.as_ref()
    }
}
