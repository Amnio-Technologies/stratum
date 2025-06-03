use super::{obj_tree_viewer, property_editor};
use crate::state::UiState;

pub fn draw(ui: &mut egui::Ui, ui_state: &mut UiState) {
    ui.horizontal(|ui| {
        if ui
            .selectable_label(ui_state.element_select_active, "👆")
            .clicked()
        {
            ui_state.element_select_active = !ui_state.element_select_active;
        }

        ui.selectable_label(false, "💡");
    });
    ui.separator();
    obj_tree_viewer::draw(ui, ui_state);
    ui.separator();
    property_editor::draw(ui, ui_state);
}
