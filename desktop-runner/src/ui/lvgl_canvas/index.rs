use super::render::draw_lvgl_canvas;
use crate::{state::UiState, stratum_lvgl_ui::StratumLvglUI};
use egui::{CentralPanel, Color32, Context, Direction, Frame, Layout};

pub fn draw(ctx: &Context, ui_state: &mut UiState, lvgl_ui: &mut StratumLvglUI) {
    CentralPanel::default()
        .frame(Frame::central_panel(&ctx.style()).fill(Color32::from_rgb(20, 20, 20)))
        .show(ctx, |ui| {
            ui.with_layout(
                Layout::centered_and_justified(Direction::LeftToRight),
                |ui| {
                    let tex = lvgl_ui.update(ctx);
                    let rects = lvgl_ui.flush_collector.active_events();

                    draw_lvgl_canvas(ui, ui_state, tex, rects);
                },
            );
        });
}
