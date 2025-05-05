use egui::Context;
use egui_sdl2_gl::{DpiScaling, EguiStateHandler, ShaderVersion};
use sdl2::{
    video::{GLContext, GLProfile, Window},
    EventPump, Sdl, VideoSubsystem,
};

use crate::debug_ui::DEBUG_UI_WIDTH;
use stratum_ui_common::amnio_bindings;

/// Initializes SDL2 and its video subsystem.
pub fn initialize_sdl() -> (Sdl, VideoSubsystem) {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    (sdl_context, video_subsystem)
}

/// Configures OpenGL attributes.
pub fn setup_gl_attr(video_subsystem: &mut VideoSubsystem) {
    let gl_attr = video_subsystem.gl_attr();
    gl_attr.set_context_profile(GLProfile::Core);
    gl_attr.set_double_buffer(true);
    gl_attr.set_multisample_samples(4);
}

/// Initializes the SDL2 window and OpenGL context.
pub fn initialize_window(video_subsystem: &mut VideoSubsystem) -> (Window, GLContext) {
    let window_width = unsafe { amnio_bindings::get_lvgl_display_width() } + DEBUG_UI_WIDTH;
    let window_height = unsafe { amnio_bindings::get_lvgl_display_height() };

    let window = video_subsystem
        .window("amnIO Stratum UI Debugger", window_width, window_height)
        .opengl()
        .resizable()
        .build()
        .unwrap();

    let gl_context = window.gl_create_context().unwrap(); // Keeps OpenGL context alive
    window.gl_make_current(&gl_context).unwrap(); // Ensures it’s the active context
    egui_sdl2_gl::gl::load_with(|s| video_subsystem.gl_get_proc_address(s) as *const _); // Loads OpenGL functions

    (window, gl_context)
}

/// Initializes egui and SDL2 event handling.
pub fn initialize_egui(
    window: &Window,
    sdl_context: Sdl,
) -> (
    egui_sdl2_gl::painter::Painter,
    EguiStateHandler,
    Context,
    EventPump,
) {
    let shader_ver = ShaderVersion::Default;
    let (painter, egui_state) = egui_sdl2_gl::with_sdl2(window, shader_ver, DpiScaling::Default);
    let egui_ctx = egui::Context::default();
    let event_pump = sdl_context.event_pump().unwrap();
    re_ui::apply_style_and_install_loaders(&egui_ctx);

    (painter, egui_state, egui_ctx, event_pump)
}
