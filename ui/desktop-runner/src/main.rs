use egui::Context;
use egui_sdl2_gl::egui::FullOutput;
use egui_sdl2_gl::painter::Painter;
use egui_sdl2_gl::{gl, sdl2};
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::video::Window;
use sdl2::EventPump;
use tracing::info;
use tracing_subscriber;

mod amnio_bindings {
    #![allow(non_camel_case_types, non_snake_case, non_upper_case_globals)]

    #[cfg(windows)]
    include!(concat!(env!("OUT_DIR"), "\\bindings.rs"));

    #[cfg(unix)]
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

mod amnio_lvgl_ui;
mod debug_ui;
mod window_init;

use amnio_lvgl_ui::AmnioLvglUI;
use window_init::{initialize_egui, initialize_sdl, initialize_window, setup_gl_attr};

use egui_sdl2_gl::EguiStateHandler;

struct UiState {
    enable_vsync: bool,
    quit: bool,
    slider_value: f64,
    debug_text: String,
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
    true
}

fn render_frame(
    window: &Window,
    painter: &mut Painter,
    egui_ctx: &Context,
    egui_state: &mut EguiStateHandler,
    ui_state: &mut UiState,
    lvgl_ui: &mut AmnioLvglUI,
) {
    unsafe {
        gl::ClearColor(0.15, 0.15, 0.15, 1.0);
        gl::Clear(gl::COLOR_BUFFER_BIT);
    }

    let ui_texture = lvgl_ui.update(egui_ctx);

    egui_ctx.begin_pass(egui_state.input.take());

    egui::SidePanel::left("lvgl_canvas")
        .resizable(false)
        .show(egui_ctx, |ui| {
            if let Some(texture) = ui_texture {
                ui.image(texture);
            } else {
                ui.label("🛑 No LVGL Texture Found");
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
    tracing_subscriber::fmt::init();

    let (sdl_context, mut video_subsystem) = initialize_sdl();
    setup_gl_attr(&mut video_subsystem);

    let (window, _context) = initialize_window(&mut video_subsystem);

    let (mut painter, mut egui_state, egui_ctx, mut event_pump) =
        initialize_egui(&window, sdl_context);

    let mut ui_state = UiState {
        enable_vsync: false,
        quit: false,
        slider_value: 10.0,
        debug_text: "Debug output area".to_string(),
    };

    info!("Initializing LVGL...");
    let mut ui = AmnioLvglUI::new().setup();
    info!("LVGL Initialized");

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
            &mut ui,
        );
    }

    info!("Shutting down...");
}
