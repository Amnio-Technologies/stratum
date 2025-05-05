use egui_sdl2_gl::{painter::Painter, EguiStateHandler};
use sdl2::{event::Event, keyboard::Keycode, video::Window, EventPump};

use crate::state::UiState;

pub fn handle_events(
    event_pump: &mut EventPump,
    egui_state: &mut EguiStateHandler,
    window: &Window,
    painter: &mut Painter,
    _ui_state: &mut UiState,
) -> bool {
    for event in event_pump.poll_iter() {
        match event {
            Event::Quit { .. }
            | Event::KeyDown {
                keycode: Some(Keycode::Escape),
                ..
            } => return false,
            _ => egui_state.process_input(window, event, painter),
        }
    }
    true
}
