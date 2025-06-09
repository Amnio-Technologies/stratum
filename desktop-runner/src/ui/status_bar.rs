use crate::{
    state::UiState,
    stratum_lvgl_ui::StratumLvglUI,
    ui::lvgl_canvas::{ZOOM_MAX, ZOOM_MIN},
};
use egui::DragValue;

pub fn draw(ctx: &egui::Context, ui_state: &mut UiState, lvgl_ui: &StratumLvglUI) {
    egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
        ui.horizontal(|ui| {
            ui.label(format!("FPS: {:.2}", lvgl_ui.current_fps()));
            ui.separator();

            ui.label("Zoom:");

            let initial_zoom = (ui_state.canvas_view.zoom * 100.0).round() / 100.0;
            let mut zoom_pct = ui_state.canvas_view.pending_zoom.unwrap_or(initial_zoom) * 100.0;

            // 2) Draw the drag‐value widget
            let resp = ui.add(
                DragValue::new(&mut zoom_pct)
                    .range((ZOOM_MIN * 100.0)..=(ZOOM_MAX * 100.0))
                    .speed(1.0)
                    .suffix("%"),
            );

            let new_zoom = (zoom_pct as f32) / 100.0;

            // 3) While the widget has focus, stash whatever they type/drag
            if resp.has_focus() {
                ui_state.canvas_view.pending_zoom = Some(new_zoom);
            }

            // 4) Commit on drag-release OR text-entry focus-loss
            if resp.dragged() || resp.lost_focus() {
                ui_state.canvas_view.zoom = new_zoom.clamp(ZOOM_MIN, ZOOM_MAX);
                // clear the pending buffer
                ui_state.canvas_view.pending_zoom = None;
            }

            // 5) Your Reset button stays the same
            if ui
                .add_enabled(
                    (ui_state.canvas_view.zoom - 1.0).abs() > f32::EPSILON,
                    egui::Button::new("Reset"),
                )
                .clicked()
            {
                ui_state.canvas_view.reset_zoom();
                ui_state.canvas_view.reset_position();
            }
            ui.separator();

            if ui
                .add_enabled(
                    ui_state.canvas_view.offset != Default::default(),
                    egui::Button::new("Re-center"),
                )
                .clicked()
            {
                ui_state.canvas_view.reset_position()
            }

            ui.separator();

            fn format_cursor_pos(pos: Option<(usize, usize)>) -> String {
                if let Some(pos) = pos {
                    format!("({}, {})", pos.0, pos.1)
                } else {
                    "(__, __)".into()
                }
            }

            ui.label(format!("Pixel: {}", format_cursor_pos(ui_state.cursor_pos)));

            ui.separator();
        });
    });
}
