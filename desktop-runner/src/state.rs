use amnio_firmware::modules::{module_manager::ModuleManager, system_controller::SystemController};
use std::sync::Arc;
use std::time::{Duration, Instant};

pub struct UiState {
    pub enable_vsync: bool,
    pub module_manager: ModuleManager,
    pub system_controller: Arc<SystemController>,
    pub quit: bool,
    pub fps: f64,
    pub frame_counter: u32,
    pub last_fps_update: Instant,
}

impl UiState {
    pub fn new() -> Self {
        Self {
            enable_vsync: false,
            module_manager: ModuleManager::new(),
            system_controller: SystemController::new(),
            quit: false,
            fps: 0.0,
            frame_counter: 0,
            last_fps_update: Instant::now(),
        }
    }
}

pub fn update_fps(ui_state: &mut UiState, start_time: &Instant) {
    ui_state.frame_counter += 1;
    let elapsed = ui_state.last_fps_update.elapsed();
    if elapsed >= Duration::from_secs(1) {
        ui_state.fps = ui_state.frame_counter as f64 / elapsed.as_secs_f64();
        ui_state.frame_counter = 0;
        ui_state.last_fps_update = Instant::now();
    }

    let frame_duration = start_time.elapsed();
    if frame_duration < Duration::from_millis(16) {
        std::thread::sleep(Duration::from_millis(16) - frame_duration);
    }
}
