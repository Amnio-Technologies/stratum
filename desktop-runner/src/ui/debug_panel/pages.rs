use strum_macros::EnumIter;

use crate::{state::UiState, ui::debug_panel::logs_page};

use super::{
    elements_page::{self, property_editor::PropertyEditorTabs},
    performance_page, ui_build_page,
};

#[derive(Debug, Clone, PartialEq, EnumIter)]
pub enum DebugSidebarPages {
    UiBuild,
    Elements(PropertyEditorTabs),
    Logs,
    Performance,
}

impl DebugSidebarPages {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::UiBuild => "UI Build",
            Self::Elements(_) => "Elements",
            Self::Logs => "Logs",
            Self::Performance => "Performance",
        }
    }

    pub fn draw_debug_page(&self, ui: &mut egui::Ui, ui_state: &mut UiState) {
        match self {
            Self::UiBuild => ui_build_page::draw(ui, ui_state),
            Self::Elements(_) => elements_page::draw(ui, ui_state),
            Self::Logs => logs_page::draw(ui, ui_state),
            Self::Performance => performance_page::draw(ui, ui_state),
        }
    }
}
