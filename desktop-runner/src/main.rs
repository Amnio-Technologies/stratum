// main.rs

mod fps_tracker;
mod hot_reload_manager;
mod icon_manager;
mod lvgl_backend;
mod lvgl_obj_tree;
mod state;
mod stratum_app;
mod stratum_lvgl_ui;
mod ui;

fn main() {
    eframe::run_native(
        "Amnio LVScope",
        eframe::NativeOptions::default(),
        Box::new(move |cc| {
            // Pass the logger through into your app
            Ok(Box::new(stratum_app::StratumApp::new(cc)))
        }),
    )
    .unwrap();
}
