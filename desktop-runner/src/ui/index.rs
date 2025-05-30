use super::{debug_panel, lvgl_canvas, menu_bar, status_bar};
use crate::{state::UiState, stratum_lvgl_ui::StratumLvglUI};

pub fn draw_ui(ctx: &egui::Context, ui_state: &mut UiState, lvgl_ui: &mut StratumLvglUI) {
    // Have the status bar occupy the entirety of the window's width
    status_bar::draw(ctx, ui_state);
    // Draw the debug panel first so it eats up space and gives the canvas renderer and menu bar the leftovers
    debug_panel::draw(ctx, ui_state);

    menu_bar::draw(ctx, ui_state);
    lvgl_canvas::draw(ctx, ui_state, lvgl_ui);
}
