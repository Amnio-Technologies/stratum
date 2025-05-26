use std::pin::Pin;

use egui::{ColorImage, TextureHandle, TextureOptions};
use stratum_ui_common::{amnio_bindings, lvgl_backend::LvglBackend};

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
        let (width, height) = unsafe {
            (
                amnio_bindings::get_lvgl_display_width() as usize,
                amnio_bindings::get_lvgl_display_height() as usize,
            )
        };

        let mut rgba_data = vec![0u8; width * height * 4];

        for (i, &pixel) in frame_buffer.iter().enumerate() {
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
