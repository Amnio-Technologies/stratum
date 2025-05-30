use strum::IntoEnumIterator;

use crate::state::UiState;

use super::pages::DebugSidebarPages;

pub fn create_debug_ui(ui: &mut egui::Ui, ui_state: &mut UiState) {
    ui.horizontal(|ui| {
        for page in DebugSidebarPages::iter() {
            if ui
                .selectable_label(ui_state.selected_debug_page == page, page.as_str())
                .clicked()
            {
                ui_state.selected_debug_page = page;
            }
        }
    });

    ui.separator();

    let selected_page = ui_state.selected_debug_page.clone();
    selected_page.draw_debug_page(ui, ui_state);
}
