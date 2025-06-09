use strum::IntoEnumIterator;
use strum_macros::EnumIter;

use crate::{state::UiState, ui::debug_panel::pages::DebugSidebarPages};

#[derive(Debug, Clone, Copy, PartialEq, EnumIter)]
pub enum PropertyEditorTabs {
    BaseProperties,
    StyleProperties,
    Events,
}

fn draw_base_properties_editor_tab(ui: &mut egui::Ui, ui_state: &UiState) {}

fn draw_style_properties_editor_tab(ui: &mut egui::Ui, ui_state: &UiState) {}

fn draw_events_editor_tab(ui: &mut egui::Ui, ui_state: &UiState) {}

impl PropertyEditorTabs {
    pub fn as_str(&self) -> &'static str {
        match self {
            PropertyEditorTabs::BaseProperties => "Base",
            PropertyEditorTabs::StyleProperties => "Style",
            PropertyEditorTabs::Events => "Events",
        }
    }

    pub fn draw_debug_page(&self, ui: &mut egui::Ui, ui_state: &mut UiState) {
        match self {
            PropertyEditorTabs::BaseProperties => draw_base_properties_editor_tab(ui, ui_state),
            PropertyEditorTabs::StyleProperties => draw_style_properties_editor_tab(ui, ui_state),
            PropertyEditorTabs::Events => draw_events_editor_tab(ui, ui_state),
        }
    }
}

impl Default for PropertyEditorTabs {
    fn default() -> Self {
        Self::BaseProperties
    }
}

pub fn draw(ui: &mut egui::Ui, ui_state: &mut UiState) {
    if let DebugSidebarPages::Elements(selected_tab) = ui_state.selected_debug_page {
        // 1) Collect all the tab variants
        let tabs: Vec<_> = PropertyEditorTabs::iter().collect();
        let count = tabs.len() as f32;

        // 2) Compute how wide each tab should be:
        let spacing = ui.spacing().item_spacing.x;
        let total_spacing = spacing * (count - 1.0);
        let avail = ui.available_width();
        let tab_width = (avail - total_spacing) / count;

        // 3) Lay them out in a horizontal row, each sized to `tab_width`
        ui.horizontal_top(|ui| {
            for &tab in &tabs {
                let is_selected = tab == selected_tab;
                let lbl = egui::SelectableLabel::new(is_selected, tab.as_str());
                // height = 0.0 → use default interact height
                let resp = ui.add_sized([tab_width, 0.0], lbl);
                if resp.clicked() {
                    ui_state.selected_debug_page = DebugSidebarPages::Elements(tab);
                }
            }
        });

        selected_tab.draw_debug_page(ui, ui_state);
    }
}
