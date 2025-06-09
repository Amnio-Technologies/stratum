use crate::ui::debug_panel::performance_page::LvglFpsLimit;
// state.rs
use crate::hot_reload_manager::SharedHotReloadManager;
use crate::icon_manager::IconManager;
use crate::lvgl_obj_tree::SharedTreeManager;
use crate::ui::debug_panel::pages::DebugSidebarPages;
use crate::ui::lvgl_canvas::CanvasView;
use std::path::PathBuf;
use std::sync::Arc;
use stratum_firmware_common::modules::{
    module_manager::ModuleManager, system_controller::SystemController,
};
use stratum_ui_common::ui_logging::UiLogger;

/// Holds global UI state, including the LVGL renderer, modules, and logs.
pub struct UiState {
    pub module_manager: ModuleManager,
    pub system_controller: Arc<SystemController>,
    /// Logger for UI messages (forwarded from C).
    pub ui_logger: Arc<UiLogger>,

    pub hot_reload_manager: SharedHotReloadManager,

    pub tree_manager: SharedTreeManager,

    /// Accumulated lines for debug display.
    pub log_buffer: Vec<String>,

    pub selected_build: Option<PathBuf>,
    pub selected_debug_page: DebugSidebarPages,
    pub canvas_view: CanvasView,
    pub icon_manager: IconManager,
    pub cursor_pos: Option<(usize, usize)>,
    pub element_select_active: bool,
    pub lvgl_fps_limit: LvglFpsLimit,
}

impl UiState {
    /// Create a new UiState, registering the UI logger and initializing fields.
    pub fn new(
        ui_logger: Arc<UiLogger>,
        hot_reload_manager: SharedHotReloadManager,
        tree_manager: SharedTreeManager,
        icon_manager: IconManager,
    ) -> Self {
        UiState {
            module_manager: ModuleManager::new(),
            system_controller: SystemController::new(),
            ui_logger,
            hot_reload_manager,
            tree_manager,
            log_buffer: Vec::new(),
            selected_build: None,
            selected_debug_page: DebugSidebarPages::UiBuild,
            canvas_view: CanvasView::default(),
            icon_manager,
            cursor_pos: None,
            element_select_active: false,
            lvgl_fps_limit: LvglFpsLimit::Preset(30),
        }
    }
}
