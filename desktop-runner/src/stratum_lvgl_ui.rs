use egui::{ColorImage, TextureHandle, TextureOptions};
use stratum_ui_common::lvgl_backend::LvglBackend;

use crate::lvgl_backend::DesktopLvglBackend;

static DESKTOP_LVGL_BACKEND: DesktopLvglBackend = DesktopLvglBackend;

pub struct StratumLvglUI {
    lvgl_renderer: LvglRenderer,
}

impl StratumLvglUI {
    pub fn new() -> Self {
        DESKTOP_LVGL_BACKEND.setup_ui();

        Self {
            lvgl_renderer: LvglRenderer::new(),
        }
    }

    pub fn update(&mut self, egui_ctx: &egui::Context) -> Option<&TextureHandle> {
        DESKTOP_LVGL_BACKEND.update_ui();
        self.lvgl_renderer.update_lvgl_framebuffer(egui_ctx);

        self.lvgl_renderer.get_texture()
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
    fn update_lvgl_framebuffer(&mut self, egui_ctx: &egui::Context) {
        let (fb, width, height) = match DESKTOP_LVGL_BACKEND.get_framebuffer() {
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
