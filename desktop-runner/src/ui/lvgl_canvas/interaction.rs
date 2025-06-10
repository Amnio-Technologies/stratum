use egui::{Event, PointerButton, Rect, Sense, Ui};
use stratum_ui_common::stratum_ui_ffi;

use crate::lvgl_obj_tree::TreeManager;
use crate::state::UiState;
use crate::stratum_lvgl_ui::RENDER_LOCK;

pub fn clear_element_selection_on_background_click(
    ui: &mut egui::Ui,
    ui_state: &mut UiState,
    avail: Rect,
) {
    if ui
        .interact(avail, ui.id().with("canvas_bg"), Sense::click())
        .clicked()
    {
        ui_state
            .tree_manager
            .lock()
            .unwrap()
            .tree_state
            .set_selected(vec![]);
    }
}

pub fn handle_canvas_click_selection(ui: &mut Ui, ui_state: &mut UiState, view_rect: Rect) {
    for event in ui.ctx().input(|i| i.events.clone()) {
        if let Event::PointerButton {
            pos,
            button: PointerButton::Primary,
            pressed: true,
            ..
        } = event
        {
            if view_rect.contains(pos) && ui_state.element_select_active {
                if let Some((lvgl_x, lvgl_y)) = ui_state.cursor_pos {
                    TreeManager::request_obj_at_point(&ui_state.tree_manager, lvgl_x, lvgl_y);
                }
                ui_state.element_select_active = false;
                let _guard = RENDER_LOCK.lock().unwrap();
                unsafe {
                    stratum_ui_ffi::revert_clickability();
                }
            }
        }
    }
}
