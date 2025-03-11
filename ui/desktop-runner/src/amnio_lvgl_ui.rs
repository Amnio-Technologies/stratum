use crate::amnio_bindings;
use egui::{ColorImage, TextureHandle, TextureOptions};

struct LvglRenderer {
    texture: Option<TextureHandle>,
}

impl LvglRenderer {
    fn new() -> Self {
        LvglRenderer { texture: None }
    }

    /// Converts LVGL's RGB565 framebuffer to RGBA and uploads to GPU
    fn update_lvgl_framebuffer(&mut self, egui_ctx: &egui::Context) {
        let (fb, width, height) = match LvglEnvironment::get_framebuffer() {
            Some(fb) => fb,
            None => {
                eprintln!("🛑 LVGL framebuffer is null! Skipping update.");
                return;
            }
        };

        let mut rgba_data = vec![0u8; width * height * 4];

        for (i, &pixel) in fb.iter().enumerate() {
            let r = ((pixel >> 11) & 0x1F) << 3;
            let g = ((pixel >> 5) & 0x3F) << 2;
            let b = (pixel & 0x1F) << 3;
            rgba_data[i * 4] = r as u8;
            rgba_data[i * 4 + 1] = g as u8;
            rgba_data[i * 4 + 2] = b as u8;
            rgba_data[i * 4 + 3] = 255 as u8;
        }

        let color_image = ColorImage::from_rgba_unmultiplied([width, height], &rgba_data);
        self.texture =
            Some(egui_ctx.load_texture("lvgl_fb", color_image, TextureOptions::default()));
    }

    fn get_texture(&self) -> Option<&TextureHandle> {
        self.texture.as_ref()
    }
}

struct LvglEnvironment;
impl LvglEnvironment {
    fn update_ui() {
        unsafe { amnio_bindings::lvgl_update() };
    }

    fn setup_ui() {
        unsafe {
            amnio_bindings::lvgl_setup();
        }
    }

    fn get_framebuffer() -> Option<(&'static mut [u16], usize, usize)> {
        let ptr = unsafe { amnio_bindings::get_lvgl_framebuffer() };
        if ptr.is_null() {
            return None;
        }

        let width = unsafe { amnio_bindings::get_lvgl_display_width() } as usize;
        let height = unsafe { amnio_bindings::get_lvgl_display_height() } as usize;

        unsafe {
            Some((
                std::slice::from_raw_parts_mut(ptr, width * height),
                width,
                height,
            ))
        }
    }
}

pub struct AmnioLvglUI {
    lvgl_renderer: LvglRenderer,
}

impl AmnioLvglUI {
    pub fn new() -> Self {
        Self {
            lvgl_renderer: LvglRenderer::new(),
        }
    }

    pub fn update(&mut self, egui_ctx: &egui::Context) -> Option<&TextureHandle> {
        LvglEnvironment::update_ui();
        self.lvgl_renderer.update_lvgl_framebuffer(egui_ctx);

        self.lvgl_renderer.get_texture()
    }

    pub fn setup(self) -> Self {
        LvglEnvironment::setup_ui();

        self
    }
}
