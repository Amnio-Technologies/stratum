use egui::{ColorImage, TextureHandle, TextureOptions};
use std::sync::Mutex;

const LVGL_SCREEN_WIDTH: usize = 480;
const LVGL_SCREEN_HEIGHT: usize = 320;

// Shared framebuffer memory (sync with LVGL's framebuffer)
static LVGL_FRAMEBUFFER: Mutex<[u16; LVGL_SCREEN_WIDTH * LVGL_SCREEN_HEIGHT]> =
    Mutex::new([0; LVGL_SCREEN_WIDTH * LVGL_SCREEN_HEIGHT]);

pub struct LvglRenderer {
    texture: Option<TextureHandle>,
}

impl LvglRenderer {
    pub fn new() -> Self {
        LvglRenderer { texture: None }
    }

    /// Converts LVGL's RGB565 framebuffer to RGBA and uploads to GPU
    pub fn update_lvgl_framebuffer(&mut self, egui_ctx: &egui::Context) {
        let fb = LVGL_FRAMEBUFFER.lock().unwrap();
        let mut rgba_data = vec![0u8; LVGL_SCREEN_WIDTH * LVGL_SCREEN_HEIGHT * 4];

        for (i, &pixel) in fb.iter().enumerate() {
            let r = ((pixel >> 11) & 0x1F) << 3;
            let g = ((pixel >> 5) & 0x3F) << 2;
            let b = (pixel & 0x1F) << 3;
            rgba_data[i * 4] = r as u8;
            rgba_data[i * 4 + 1] = g as u8;
            rgba_data[i * 4 + 2] = b as u8;
            rgba_data[i * 4 + 3] = 255 as u8;
        }

        let color_image =
            ColorImage::from_rgba_unmultiplied([LVGL_SCREEN_WIDTH, LVGL_SCREEN_HEIGHT], &rgba_data);
        self.texture =
            Some(egui_ctx.load_texture("lvgl_fb", color_image, TextureOptions::default()));
    }

    /// Returns the `egui` texture handle
    pub fn get_texture(&self) -> Option<&TextureHandle> {
        self.texture.as_ref()
    }
}
