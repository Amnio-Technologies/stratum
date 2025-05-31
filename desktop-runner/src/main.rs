// main.rs

mod hot_reload_manager;
mod icon;
mod lvgl_backend;
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
