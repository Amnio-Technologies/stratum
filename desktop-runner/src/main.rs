mod debug_ui;
mod lvgl_backend;
mod state;
mod stratum_app;
mod stratum_lvgl_ui;

fn main() {
    env_logger::init();
    stratum_ui_common::ui_logging::start_ui_log_worker();

    eframe::run_native(
        "amnIO Stratum Simulator",
        eframe::NativeOptions::default(),
        Box::new(|cc| Ok(Box::new(stratum_app::StratumApp::new(cc)))),
    );
}
