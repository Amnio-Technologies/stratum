// main.rs

mod debug_panel;
mod hot_reload_manager;
mod lvgl_backend;
mod state;
mod stratum_app;
mod stratum_lvgl_ui;

use hot_reload_manager::HotReloadManager;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use stratum_ui_common::lvgl_obj_tree::TreeManager;
use stratum_ui_common::ui_logging::UiLogger;

fn main() {
    let hot_reload_manager = Arc::new(Mutex::new(HotReloadManager::new(
        PathBuf::from("../stratum-ui/build/desktop/libstratum-ui.dll"),
        PathBuf::from("../stratum-ui/build.py"),
        vec![
            PathBuf::from("../stratum-ui/src"),
            PathBuf::from("../stratum-ui/include"),
        ],
    )));

    env_logger::init();
    HotReloadManager::start(hot_reload_manager.clone());
    let tree_manager = TreeManager::new();

    // Spin up our C -> Rust logger with a 10_000-entry cap
    let ui_logger: Arc<UiLogger> = UiLogger::new(10_000);

    eframe::run_native(
        "Amnio LVScope",
        eframe::NativeOptions::default(),
        Box::new(move |cc| {
            // Pass the logger through into your app
            Ok(Box::new(stratum_app::StratumApp::new(
                cc,
                ui_logger.clone(),
                hot_reload_manager,
                tree_manager,
            )))
        }),
    )
    .unwrap();
}
