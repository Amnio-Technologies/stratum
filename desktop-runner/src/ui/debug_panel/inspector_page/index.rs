use super::{obj_tree_viewer, property_editor};
use crate::state::UiState;

pub fn draw(ui: &mut egui::Ui, ui_state: &mut UiState) {
    ui.horizontal(|ui| {
        if ui
            .selectable_label(ui_state.element_select_active, "👆")
            .clicked()
        {
            ui_state.element_select_active = !ui_state.element_select_active;
            // TODO: lock the tree with our RenderLock(Arc<Mutex<()>>), capture the current clickable state of all objects in the tree, save it somewhere else,
            // apply the clickable state to everything, once user makes selection or element_select_active goes back to false, we re-capture the RenderLock,
            // and re-apply the cached state for the nodes that we changed its clickability
        }

        ui.selectable_label(false, "💡");
    });
    ui.separator();
    obj_tree_viewer::draw(ui, ui_state);
    ui.separator();
    property_editor::draw(ui, ui_state);
}
