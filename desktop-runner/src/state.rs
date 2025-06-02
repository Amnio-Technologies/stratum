// state.rs
use crate::hot_reload_manager::SharedHotReloadManager;
use crate::icon_manager::IconManager;
use crate::lvgl_obj_tree::SharedTreeManager;
use crate::ui::debug_panel::pages::DebugSidebarPages;
use egui::Vec2;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};
use stratum_firmware_common::modules::{
    module_manager::ModuleManager, system_controller::SystemController,
};
use stratum_ui_common::ui_logging::UiLogger;

pub struct CanvasView {
    pub zoom: f32,
    pub offset: Vec2,
    pub pending_zoom: Option<f32>,
}

impl CanvasView {
    pub fn reset_zoom(&mut self) {
        self.zoom = 1.0;
        self.pending_zoom = None;
    }
    pub fn reset_position(&mut self) {
        self.offset = Default::default();
    }
}

impl Default for CanvasView {
    fn default() -> Self {
        Self {
            zoom: 1.0,
            offset: Default::default(),
            pending_zoom: None,
        }
    }
}

/// Holds global UI state, including the LVGL renderer, modules, and logs.
pub struct UiState {
    pub enable_vsync: bool,
    pub module_manager: ModuleManager,
    pub system_controller: Arc<SystemController>,
    pub fps: f64,
    pub frame_counter: u32,
    pub last_fps_update: Instant,

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
            enable_vsync: false,
            module_manager: ModuleManager::new(),
            system_controller: SystemController::new(),
            fps: 0.0,
            frame_counter: 0,
            last_fps_update: Instant::now(),
            ui_logger,
            hot_reload_manager,
            tree_manager,
            log_buffer: Vec::new(),
            selected_build: None,
            selected_debug_page: DebugSidebarPages::UiBuild,
            canvas_view: CanvasView::default(),
            icon_manager,
            cursor_pos: None,
        }
    }
}

/// Updates the FPS counter and enforces a ~60Hz sleep if vsync is disabled.
pub fn update_fps(ui_state: &mut UiState, start_time: &Instant) {
    ui_state.frame_counter += 1;
    let elapsed = ui_state.last_fps_update.elapsed();
    if elapsed >= Duration::from_secs(1) {
        ui_state.fps = ui_state.frame_counter as f64 / elapsed.as_secs_f64();
        ui_state.frame_counter = 0;
        ui_state.last_fps_update = Instant::now();
    }

    // Simple frame limiter (~60 FPS)
    let frame_duration = start_time.elapsed();
    if frame_duration < Duration::from_millis(16) && !ui_state.enable_vsync {
        std::thread::sleep(Duration::from_millis(16) - frame_duration);
    }
}
