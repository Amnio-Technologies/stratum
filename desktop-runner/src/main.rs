// main.rs

mod debug_ui;
mod hot_reload_manager;
mod lvgl_backend;
mod state;
mod stratum_app;
mod stratum_lvgl_ui;

use hot_reload_manager::HotReloadManager;
use std::sync::Arc;
use std::{path::PathBuf, time::Duration};
use stratum_ui_common::{amnio_bindings, ui_logging::UiLogger};
fn main() {
    env_logger::init();

    // Spin up our C→Rust logger with a 10_000-entry cap
    let ui_logger: Arc<UiLogger> = UiLogger::new(10_000);
    let manager = HotReloadManager::new(
        PathBuf::from("../stratum-ui/build/desktop/libstratum-ui.dll"),
        PathBuf::from("../stratum-ui/build.py"),
        vec![
            PathBuf::from("../stratum-ui/src"),
            PathBuf::from("../stratum-ui/include"),
        ],
        Duration::from_millis(300),
    );

    manager.start();

    unsafe {
        amnio_bindings::init_dynamic_bindings("../stratum-ui/build/desktop/libstratum-ui.dll");
    };

    eframe::run_native(
        "amnIO Stratum Simulator",
        eframe::NativeOptions::default(),
        Box::new(move |cc| {
            // Pass the logger through into your app
            Ok(Box::new(stratum_app::StratumApp::new(
                cc,
                ui_logger.clone(),
            )))
        }),
    )
    .unwrap();
}
