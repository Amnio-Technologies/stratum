use stratum_ui_common::stratum_ui_ffi;

use super::{obj_tree_viewer, property_editor};
use crate::{state::UiState, stratum_lvgl_ui::RENDER_LOCK};

pub fn draw(ui: &mut egui::Ui, ui_state: &mut UiState) {
    ui.horizontal(|ui| {
        if ui
            .selectable_label(ui_state.element_select_active, "👆")
            .on_hover_text("Toggle element selection mode")
            .clicked()
        {
            ui_state.element_select_active = !ui_state.element_select_active;

            let _guard = RENDER_LOCK.lock().unwrap();

            unsafe {
                if ui_state.element_select_active {
                    stratum_ui_ffi::make_all_clickable();
                } else {
                    stratum_ui_ffi::revert_clickability();
                }
            }
        }

        if ui
            .selectable_label(ui_state.repaint_flash_active, "💡")
            .on_hover_text("Flash on repaint")
            .clicked()
        {
            ui_state.repaint_flash_active = !ui_state.repaint_flash_active;
        }
    });
    ui.separator();
    obj_tree_viewer::draw(ui, ui_state);
    ui.separator();
    property_editor::draw(ui, ui_state);
}
