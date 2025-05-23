use eframe::{egui, CreationContext, Frame};
use egui::TextureHandle;

use crate::{
    debug_ui,
    state::{update_fps, UiState},
    stratum_lvgl_ui::StratumLvglUI,
};

pub struct StratumApp {
    ui_state: UiState,
    lvgl_ui: StratumLvglUI,
    lvgl_tex: Option<TextureHandle>,
    last_frame_start: std::time::Instant,
}

impl StratumApp {
    pub fn new(_cc: &CreationContext<'_>) -> Self {
        Self {
            ui_state: UiState::new(),
            lvgl_ui: StratumLvglUI::new(),
            lvgl_tex: None,
            last_frame_start: std::time::Instant::now(),
        }
    }
}

impl eframe::App for StratumApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut Frame) {
        // ――― update LVGL & grab texture ―――
        self.lvgl_tex = self.lvgl_ui.update(ctx).cloned();

        // ――― build UI ―――
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                draw_lvgl_canvas(ui, self.lvgl_tex.as_ref());
                draw_debug_panel(ui, &mut self.ui_state);
            });
        });

        // ――― FPS calc & continuous repaint ―――
        update_fps(&mut self.ui_state, &self.last_frame_start);
        self.last_frame_start = std::time::Instant::now();
        ctx.request_repaint(); // keep it real-time
    }
}

// ---------- helpers ----------

fn draw_lvgl_canvas(ui: &mut egui::Ui, tex: Option<&TextureHandle>) {
    let width = unsafe { stratum_ui_common::amnio_bindings::get_lvgl_display_width() as f32 };
    let height = ui.available_height();

    ui.allocate_ui_with_layout(
        egui::vec2(width, height),
        egui::Layout::left_to_right(egui::Align::TOP),
        |ui| {
            if let Some(t) = tex {
                ui.image(t);
            } else {
                ui.label("No LVGL texture");
            }
        },
    );
}

fn draw_debug_panel(ui: &mut egui::Ui, state: &mut UiState) {
    ui.allocate_ui_with_layout(
        ui.available_size(),
        egui::Layout::top_down(egui::Align::Min),
        |ui| debug_ui::create_debug_ui(ui, state),
    );
}
