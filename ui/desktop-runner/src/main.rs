use std::sync::Arc;
use std::time::{Duration, Instant};

use amnio_common::ui_logging::start_ui_log_worker;
use amnio_firmware::modules::module_manager::ModuleManager;
use amnio_firmware::modules::system_controller::SystemController;
use egui::Context;
use egui_sdl2_gl::egui::FullOutput;
use egui_sdl2_gl::painter::Painter;
use egui_sdl2_gl::{gl, sdl2};
use log::info;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::video::Window;
use sdl2::EventPump;

mod amnio_bindings {
    #![allow(
        non_camel_case_types,
        non_snake_case,
        non_upper_case_globals,
        dead_code
    )]

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
    module_manager: ModuleManager,
    system_controller: Arc<SystemController>,
    quit: bool,
    fps: f64,
    frame_counter: u32, // Count frames in a second
    last_fps_update: std::time::Instant,
}

fn handle_events(
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

    egui::CentralPanel::default().show(egui_ctx, |ui| {
        ui.horizontal(|ui| {
            // LVGL Canvas
            ui.allocate_ui_with_layout(
                egui::vec2(
                    unsafe { amnio_bindings::get_lvgl_display_width() as f32 },
                    ui.available_height(),
                ),
                egui::Layout::left_to_right(egui::Align::TOP),
                |ui| {
                    if let Some(texture) = ui_texture {
                        ui.image(texture);
                    } else {
                        ui.label("No LVGL Texture Found");
                    }
                },
            );

            // Debug Panel (takes remaining space)
            ui.allocate_ui_with_layout(
                ui.available_size(),
                egui::Layout::top_down(egui::Align::Min),
                |ui| {
                    debug_ui::create_debug_ui(ui, ui_state);
                },
            );
        });
    });

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
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    start_ui_log_worker();

    let (sdl_context, mut video_subsystem) = initialize_sdl();
    setup_gl_attr(&mut video_subsystem);

    let (window, _context) = initialize_window(&mut video_subsystem);

    let (mut painter, mut egui_state, egui_ctx, mut event_pump) =
        initialize_egui(&window, sdl_context);

    let mut ui_state = UiState {
        enable_vsync: false,
        module_manager: ModuleManager::new(),
        system_controller: SystemController::new(),
        quit: false,
        fps: 0.0,
        frame_counter: 0,
        last_fps_update: Instant::now(),
    };

    info!("Initializing LVGL...");
    let mut ui = AmnioLvglUI::new().setup();
    info!("LVGL Initialized");

    while !ui_state.quit {
        let start_time = Instant::now();

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

    info!("Shutting down...");
}

fn update_fps(ui_state: &mut UiState, start_time: &Instant) {
    ui_state.frame_counter += 1;
    let elapsed_time = ui_state.last_fps_update.elapsed();
    if elapsed_time >= Duration::from_secs(1) {
        ui_state.fps = ui_state.frame_counter as f64 / elapsed_time.as_secs_f64();
        ui_state.frame_counter = 0;
        ui_state.last_fps_update = Instant::now();
    }

    // Delay to simulate ~60 FPS if needed
    let frame_duration = start_time.elapsed();
    if frame_duration < Duration::from_millis(16) {
        std::thread::sleep(Duration::from_millis(16) - frame_duration);
    }
}
