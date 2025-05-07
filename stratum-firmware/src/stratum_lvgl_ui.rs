use crate::lvgl_backend::Esp32LvglBackend;

static DESKTOP_LVGL_BACKEND: Esp32LvglBackend = Esp32LvglBackend;

// pub struct StratumLvglUI {
//     lvgl_renderer: LvglRenderer,
// }

// impl StratumLvglUI {
//     pub fn new() -> Self {
//         DESKTOP_LVGL_BACKEND.setup_ui();

//         Self {
//             lvgl_renderer: LvglRenderer::new(),
//         }
//     }

//     pub fn update(&mut self) -> Option<&TextureHandle> {
//         DESKTOP_LVGL_BACKEND.update_ui();
//         self.lvgl_renderer.update_lvgl_framebuffer(egui_ctx);

//         self.lvgl_renderer.get_texture()
//     }
// }
