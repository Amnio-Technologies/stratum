use strum_macros::EnumIter;

use crate::state::UiState;

use super::{
    inspector_page::{draw_inspector_debug_ui, property_editor::PropertyEditorTabs},
    ui_build_page::draw_uibuild_debug_ui,
};

#[derive(Debug, Clone, PartialEq, EnumIter)]
pub enum DebugSidebarPages {
    UiBuild,
    Inspector(PropertyEditorTabs),
}

impl DebugSidebarPages {
    pub fn as_str(&self) -> &'static str {
        match self {
            DebugSidebarPages::UiBuild => "UI Build",
            DebugSidebarPages::Inspector(_) => "Inspector",
        }
    }

    pub fn draw_debug_page(&self, ui: &mut egui::Ui, ui_state: &mut UiState) {
        match self {
            DebugSidebarPages::UiBuild => draw_uibuild_debug_ui(ui, ui_state),
            DebugSidebarPages::Inspector(_) => draw_inspector_debug_ui(ui, ui_state),
        }
    }
}
