use crate::{
    state::{CanvasView, UiState},
    stratum_lvgl_ui::StratumLvglUI,
};
use egui::{Color32, Direction, Layout, Pos2, Rect, Response, Sense, Stroke, TextureHandle, Vec2};
pub const ZOOM_MIN: f32 = 0.1;
pub const ZOOM_MAX: f32 = 200.0;

fn draw_lvgl_canvas(ui: &mut egui::Ui, tex: Option<&TextureHandle>, view: &mut CanvasView) {
    // 1) Figure out native LVGL size
    let display_w = unsafe { stratum_ui_common::stratum_ui_ffi::get_lvgl_display_width() as f32 };
    let display_h = unsafe { stratum_ui_common::stratum_ui_ffi::get_lvgl_display_height() as f32 };

    // 2) Grab available rect once
    let avail = ui.available_rect_before_wrap();

    // 3) Handle scroll‐to‐zoom BEFORE computing rect so everything
    //    uses the new zoom/offset in the same frame
    if let Some(cursor) = ui
        .ctx()
        .input(|i| i.pointer.hover_pos())
        .filter(|pos| avail.contains(*pos))
    {
        let scroll = ui.ctx().input(|i| i.raw_scroll_delta.y);
        if scroll != 0.0 {
            // compute new zoom
            let old_zoom = view.zoom;
            let new_zoom = (old_zoom * (1.0 + scroll * 0.001)).clamp(ZOOM_MIN, ZOOM_MAX);
            view.zoom = new_zoom;

            // figure out which LVGL‐pixel was under cursor before zoom
            let world_pixel = (cursor
                - (avail.center() + view.offset
                    - Vec2::new(display_w, display_h) * old_zoom * 0.5))
                / old_zoom;

            // after zoom, we want world_pixel to still sit under cursor
            let scaled = Vec2::new(display_w, display_h) * new_zoom;
            let new_top_left = cursor - world_pixel * new_zoom;
            view.offset = new_top_left + scaled * 0.5 - avail.center();
        }
    }

    // 4) Compute scaled size & snapped top-left
    let scaled = Vec2::new(display_w, display_h) * view.zoom;
    let unrounded_tl = avail.center() + view.offset - scaled * 0.5;
    let top_left = unrounded_tl.round(); // snap to pixel grid
    let rect = Rect::from_min_size(top_left, scaled);

    // 5) Allocate for both hover (zoom) and drag sense
    let response: Response = ui.allocate_rect(rect, Sense::click_and_drag());

    // 6) Panning (drag)
    if response.dragged() {
        view.offset += response.drag_delta();
    }

    // 7) Draw the texture (hover/drag highlight)
    if let (Some(tex), true) = (tex, response.hovered() || response.dragged()) {
        ui.painter().image(
            tex.id(),
            rect,
            Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
            egui::Color32::WHITE,
        );
    } else if let Some(tex) = tex {
        // normal draw
        ui.painter().image(
            tex.id(),
            rect,
            Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
            egui::Color32::WHITE,
        );
    } else {
        ui.painter().text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            "No LVGL texture",
            egui::FontId::proportional(16.0),
            egui::Color32::GRAY,
        );
    }

    // 8) Draw pixel grid at high zoom
    if view.zoom > 8.0 {
        let painter = ui.painter();
        let stroke = Stroke::new(1.0, Color32::from_rgb(40, 40, 40));

        let cols = (display_w as usize) + 1;
        for col in 0..=cols {
            let x = (rect.left() + (col as f32) * view.zoom).round();
            painter.line_segment(
                [Pos2::new(x, rect.top()), Pos2::new(x, rect.bottom())],
                stroke,
            );
        }

        let rows = (display_h as usize) + 1;
        for row in 0..=rows {
            let y = (rect.top() + (row as f32) * view.zoom).round();
            painter.line_segment(
                [Pos2::new(rect.left(), y), Pos2::new(rect.right(), y)],
                stroke,
            );
        }
    }
}

pub fn draw(ctx: &egui::Context, ui_state: &mut UiState, lvgl_ui: &mut StratumLvglUI) {
    egui::CentralPanel::default()
        .frame(egui::Frame::central_panel(&ctx.style()).fill(egui::Color32::from_rgb(20, 20, 20)))
        .show(ctx, |ui| {
            ui.with_layout(
                Layout::centered_and_justified(Direction::LeftToRight),
                |ui| {
                    draw_lvgl_canvas(ui, lvgl_ui.update(ctx), &mut ui_state.canvas_view);
                },
            );
        });
}
