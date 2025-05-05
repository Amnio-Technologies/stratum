use crate::render::render_frame;
use crate::sdl2_event_handling::handle_events;
use crate::state::{update_fps, UiState};
use crate::stratum_lvgl_ui::StratumLvglUI;
use crate::window_init::{initialize_egui, initialize_sdl, initialize_window, setup_gl_attr};

pub fn run_app() {
    let (sdl_context, mut video_subsystem) = initialize_sdl();
    setup_gl_attr(&mut video_subsystem);
    let (window, _gl_ctx) = initialize_window(&mut video_subsystem);
    let (mut painter, mut egui_state, egui_ctx, mut event_pump) =
        initialize_egui(&window, sdl_context);

    let mut ui_state = UiState::new();

    let mut ui = StratumLvglUI::new();

    while !ui_state.quit {
        let start_time = std::time::Instant::now();

        if !handle_events(
            &mut event_pump,
            &mut egui_state,
            &window,
            &mut painter,
            &mut ui_state,
        ) {
            break;
        }

        render_frame(
            &window,
            &mut painter,
            &egui_ctx,
            &mut egui_state,
            &mut ui_state,
            &mut ui,
        );

        update_fps(&mut ui_state, &start_time);
    }
}
