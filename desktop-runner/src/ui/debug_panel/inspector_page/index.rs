use super::{obj_tree_viewer, property_editor};
use crate::state::UiState;

pub fn draw_inspector_debug_ui(ui: &mut egui::Ui, ui_state: &mut UiState) {
    ui.horizontal(|ui| {
        ui.selectable_label(true, "👆");
        ui.selectable_label(false, "💡");
    });
    ui.separator();
    obj_tree_viewer::draw(ui, ui_state);
    ui.separator();
    property_editor::draw(ui, ui_state);
}
