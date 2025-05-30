use egui::ScrollArea;
use std::mem::discriminant;
use strum::IntoEnumIterator;

use crate::state::UiState;

use super::pages::DebugSidebarPages;

fn create_debug_ui(ui: &mut egui::Ui, ui_state: &mut UiState) {
    ui.horizontal(|ui| {
        for page in DebugSidebarPages::iter() {
            if ui
                .selectable_label(
                    discriminant(&ui_state.selected_debug_page) == discriminant(&page),
                    page.as_str(),
                )
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

pub fn draw(ctx: &egui::Context, ui_state: &mut UiState) {
    egui::SidePanel::right("debug_panel")
        .resizable(true)
        .show(ctx, |ui| {
            ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    create_debug_ui(ui, ui_state);
                });
        });
}
