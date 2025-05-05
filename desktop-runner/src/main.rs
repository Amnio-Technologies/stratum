mod app;
mod debug_ui;
mod lvgl_backend;
mod render;
mod sdl2_event_handling;
mod state;
mod stratum_lvgl_ui;
mod window_init;

fn main() {
    env_logger::init();
    stratum_ui_common::ui_logging::start_ui_log_worker();
    app::run_app();
}
