use egui::Context;
use egui_backend::egui::{self, FullOutput, TextureHandle};
use egui_backend::{gl, sdl2};
use egui_sdl2_gl::painter::Painter;
use sdl2::EventPump;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::video::Window;

mod debug_ui;
mod lvgl_renderer;
mod window_init; // New module for LVGL integration

use lvgl_renderer::LvglRenderer;
use window_init::{initialize_egui, initialize_sdl, initialize_window, setup_gl_attr}; // Handles LVGL frame conversion

use egui_sdl2_gl::{self as egui_backend, EguiStateHandler};

struct UiState {
    enable_vsync: bool,
    quit: bool,
    slider_value: f64,
    debug_text: String,
    lvgl_texture: Option<TextureHandle>, // Handle for LVGL framebuffer
}

fn handle_events(
    event_pump: &mut EventPump,
    egui_state: &mut EguiStateHandler,
    window: &Window,
    painter: &mut Painter,
    ui_state: &mut UiState,
) -> bool {
    for event in event_pump.poll_iter() {
        match event {
            Event::Quit { .. }
            | Event::KeyDown {
                keycode: Some(Keycode::Escape),
                ..
            } => return false, // Quit the application
            _ => egui_state.process_input(window, event, painter),
        }
    }
    true // Continue running
}

fn render_frame(
    window: &Window,
    painter: &mut Painter,
    egui_ctx: &Context,
    egui_state: &mut EguiStateHandler,
    ui_state: &mut UiState,
    lvgl_renderer: &mut LvglRenderer, // Handles LVGL rendering updates
) {
    unsafe {
        gl::ClearColor(0.15, 0.15, 0.15, 1.0);
        gl::Clear(gl::COLOR_BUFFER_BIT);
    }

    // Update LVGL framebuffer to texture
    lvgl_renderer.update_lvgl_framebuffer(egui_ctx);

    egui_ctx.begin_pass(egui_state.input.take());

    egui::SidePanel::left("lvgl_canvas")
        .resizable(false)
        .default_width(300.0)
        .show(egui_ctx, |ui| {
            if let Some(texture) = lvgl_renderer.get_texture() {
                ui.image(texture);
            }
        });

    // Debugging UI
    egui::SidePanel::right("debug_panel")
        .resizable(true)
        .default_width(250.0)
        .show(egui_ctx, |ui| debug_ui::create_debug_ui(ui, ui_state));

    let FullOutput {
        platform_output,
        textures_delta,
        shapes,
        pixels_per_point,
        viewport_output,
    } = egui_ctx.end_pass();

    egui_state.process_output(window, &platform_output);
    let paint_jobs = egui_ctx.tessellate(shapes, pixels_per_point);
    painter.paint_jobs(None, textures_delta, paint_jobs);
    window.gl_swap_window();

    let repaint_after = viewport_output
        .get(&egui::ViewportId::ROOT)
        .expect("Missing ViewportId::ROOT")
        .repaint_delay;

    if !repaint_after.is_zero() {
        egui_ctx.request_repaint_after(repaint_after);
    }
}

fn main() {
    let (sdl_context, mut video_subsystem) = initialize_sdl();
    setup_gl_attr(&mut video_subsystem);

    let (window, _context) = initialize_window(&mut video_subsystem);

    let (mut painter, mut egui_state, egui_ctx, mut event_pump) =
        initialize_egui(&window, sdl_context);

    let mut lvgl_renderer = LvglRenderer::new(); // LVGL Renderer instance

    let mut ui_state = UiState {
        enable_vsync: false,
        quit: false,
        slider_value: 10.0,
        debug_text: "Debug output area".to_string(),
        lvgl_texture: None,
    };

    println!("🛠️ Running SDL2 + OpenGL + egui for amnIO UI Debugging");

    while !ui_state.quit {
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
            &mut lvgl_renderer,
        );
    }

    println!("🔄 Shutting down...");
}
