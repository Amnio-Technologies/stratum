use crate::{
    state::{CanvasView, UiState},
    stratum_lvgl_ui::StratumLvglUI,
};
use egui::{
    Align2, CentralPanel, Color32, Context, Direction, FontId, Frame, Layout, Pos2, Rect, Response,
    Sense, Stroke, TextureHandle, Vec2,
};
use stratum_ui_common::lvgl_obj_tree::TreeManager;

pub const ZOOM_MIN: f32 = 0.1;
pub const ZOOM_MAX: f32 = 200.0;

fn get_lvgl_display_size() -> Vec2 {
    let w = unsafe { stratum_ui_common::stratum_ui_ffi::get_lvgl_display_width() as f32 };
    let h = unsafe { stratum_ui_common::stratum_ui_ffi::get_lvgl_display_height() as f32 };
    Vec2::new(w, h)
}

fn full_uv() -> Rect {
    Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0))
}

fn handle_zoom(ui: &egui::Ui, view: &mut CanvasView, display: Vec2, avail: Rect) {
    if let Some(cursor) = ui
        .ctx()
        .input(|i| i.pointer.hover_pos())
        .filter(|pos| avail.contains(*pos))
    {
        let scroll = ui.ctx().input(|i| i.raw_scroll_delta.y);
        if scroll != 0.0 {
            let old_zoom = view.zoom;
            let new_zoom = (old_zoom * (1.0 + scroll * 0.001)).clamp(ZOOM_MIN, ZOOM_MAX);
            view.zoom = new_zoom;

            let world_pixel =
                (cursor - (avail.center() + view.offset - display * old_zoom * 0.5)) / old_zoom;

            let scaled = display * new_zoom;
            let new_top_left = cursor - world_pixel * new_zoom;
            view.offset = new_top_left + scaled * 0.5 - avail.center();
        }
    }
}

fn compute_canvas_rect(view: &CanvasView, display: Vec2, avail: Rect) -> Rect {
    let scaled = display * view.zoom;
    let top_left = (avail.center() + view.offset - scaled * 0.5).round();
    Rect::from_min_size(top_left, scaled)
}

fn update_pan(view: &mut CanvasView, response: &Response) {
    if response.dragged() {
        view.offset += response.drag_delta();
    }
}

fn draw_canvas(ui: &mut egui::Ui, tex: Option<&TextureHandle>, rect: Rect, response: &Response) {
    if let (Some(tex), true) = (tex, response.hovered() || response.dragged()) {
        ui.painter()
            .image(tex.id(), rect, full_uv(), Color32::WHITE);
    } else if let Some(tex) = tex {
        ui.painter()
            .image(tex.id(), rect, full_uv(), Color32::WHITE);
    } else {
        ui.painter().text(
            rect.center(),
            Align2::CENTER_CENTER,
            "No LVGL texture",
            FontId::proportional(16.0),
            Color32::GRAY,
        );
    }
}

fn maybe_draw_pixel_grid(ui: &mut egui::Ui, view: &CanvasView, rect: Rect, display: Vec2) {
    if view.zoom <= 8.0 {
        return;
    }
    let painter = ui.painter();
    let stroke = Stroke::new(1.0, Color32::from_rgb(40, 40, 40));

    let cols = display.x as usize;
    for col in 0..=cols {
        let x = (rect.left() + (col as f32) * view.zoom).round();
        painter.line_segment(
            [Pos2::new(x, rect.top()), Pos2::new(x, rect.bottom())],
            stroke,
        );
    }

    let rows = display.y as usize;
    for row in 0..=rows {
        let y = (rect.top() + (row as f32) * view.zoom).round();
        painter.line_segment(
            [Pos2::new(rect.left(), y), Pos2::new(rect.right(), y)],
            stroke,
        );
    }
}

fn draw_lvgl_canvas(ui: &mut egui::Ui, ui_state: &mut UiState, tex: Option<&TextureHandle>) {
    let display_size = get_lvgl_display_size();
    let available_rect = ui.available_rect_before_wrap();
    let view = &mut ui_state.canvas_view;

    handle_zoom(ui, view, display_size, available_rect);

    let rect = compute_canvas_rect(view, display_size, available_rect);
    let response = ui.allocate_rect(rect, Sense::click_and_drag());

    update_pan(view, &response);
    draw_canvas(ui, tex, rect, &response);
    maybe_draw_pixel_grid(ui, view, rect, display_size);

    let zoom = view.zoom;
    update_user_cursor_pos(ui, ui_state, rect, display_size, zoom);

    if response.clicked() {
        if let Some((lvgl_x, lvgl_y)) = ui_state.cursor_pos {
            TreeManager::request_obj_at_point(&ui_state.tree_manager, lvgl_x, lvgl_y);
        }
    }
}

fn update_user_cursor_pos(
    ui: &mut egui::Ui,
    ui_state: &mut UiState,
    rect: Rect,
    display_size: Vec2,
    zoom: f32,
) {
    if let Some(cursor_pos) = ui.ctx().input(|i| i.pointer.hover_pos()) {
        if rect.contains(cursor_pos) {
            // local offset inside the drawn rect (in “screen-pixels”)
            let local = cursor_pos - rect.min;

            let raw_x = local.x / zoom;
            let raw_y = local.y / zoom;

            let mut lvgl_x = raw_x.floor() as usize;
            let mut lvgl_y = raw_y.floor() as usize;

            lvgl_x = lvgl_x.clamp(0, display_size.x as usize - 1);
            lvgl_y = lvgl_y.clamp(0, display_size.y as usize - 1);

            ui_state.cursor_pos = Some((lvgl_x, lvgl_y));
        } else {
            ui_state.cursor_pos = None;
        }
    } else {
        ui_state.cursor_pos = None;
    }
}

pub fn draw(ctx: &Context, ui_state: &mut UiState, lvgl_ui: &mut StratumLvglUI) {
    CentralPanel::default()
        .frame(Frame::central_panel(&ctx.style()).fill(Color32::from_rgb(20, 20, 20)))
        .show(ctx, |ui| {
            ui.with_layout(
                Layout::centered_and_justified(Direction::LeftToRight),
                |ui| {
                    draw_lvgl_canvas(ui, ui_state, lvgl_ui.update(ctx));
                },
            );
        });
}
