use egui::{Context, FullOutput, TextureHandle};
use egui_sdl2_gl::{gl, painter::Painter, EguiStateHandler};
use sdl2::video::Window;
use stratum_ui_common::amnio_bindings;

use crate::{debug_ui, state::UiState, stratum_lvgl_ui::StratumLvglUI};

fn clear_screen() {
    unsafe {
        gl::ClearColor(0.15, 0.15, 0.15, 1.0);
        gl::Clear(gl::COLOR_BUFFER_BIT);
    }
}

fn draw_ui(ctx: &Context, state: &mut UiState, tex: Option<&TextureHandle>) {
    egui::CentralPanel::default().show(ctx, |ui| {
        ui.horizontal(|ui| {
            draw_lvgl_canvas(ui, tex);
            draw_debug_panel(ui, state);
        });
    });
}

fn draw_lvgl_canvas(ui: &mut egui::Ui, tex: Option<&TextureHandle>) {
    ui.allocate_ui_with_layout(
        egui::vec2(
            unsafe { amnio_bindings::get_lvgl_display_width() as f32 },
            ui.available_height(),
        ),
        egui::Layout::left_to_right(egui::Align::TOP),
        |ui| {
            if let Some(texture) = tex {
                ui.image(texture);
            } else {
                ui.label("No LVGL Texture Found");
            }
        },
    );
}

fn draw_debug_panel(ui: &mut egui::Ui, state: &mut UiState) {
    ui.allocate_ui_with_layout(
        ui.available_size(),
        egui::Layout::top_down(egui::Align::Min),
        |ui| debug_ui::create_debug_ui(ui, state),
    );
}

fn handle_output(
    window: &Window,
    painter: &mut Painter,
    egui_state: &mut EguiStateHandler,
    ctx: &Context,
    output: FullOutput,
) {
    egui_state.process_output(window, &output.platform_output);
    let paint_jobs = ctx.tessellate(output.shapes, output.pixels_per_point);
    painter.paint_jobs(None, output.textures_delta, paint_jobs);
    window.gl_swap_window();

    if let Some(delay) = output
        .viewport_output
        .get(&egui::ViewportId::ROOT)
        .map(|v| v.repaint_delay)
    {
        if delay.is_zero() {
            return;
        }

        ctx.request_repaint_after(delay);
    }
}

pub fn render_frame(
    window: &Window,
    painter: &mut Painter,
    egui_ctx: &Context,
    egui_state: &mut EguiStateHandler,
    ui_state: &mut UiState,
    lvgl_ui: &mut StratumLvglUI,
) {
    clear_screen();
    let ui_texture = lvgl_ui.update(egui_ctx);

    egui_ctx.begin_pass(egui_state.input.take());

    draw_ui(egui_ctx, ui_state, ui_texture);

    let output = egui_ctx.end_pass();
    handle_output(window, painter, egui_state, egui_ctx, output);
}
