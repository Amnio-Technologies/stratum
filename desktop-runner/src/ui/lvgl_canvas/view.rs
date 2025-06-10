use egui::{Pos2, Rect, Response, Vec2};

use crate::state::UiState;

pub(crate) const ZOOM_MIN: f32 = 0.1;
pub(crate) const ZOOM_MAX: f32 = 200.0;

pub(crate) struct CanvasView {
    pub zoom: f32,
    pub offset: Vec2,
    pub pending_zoom: Option<f32>,
}

impl CanvasView {
    pub(crate) fn reset_zoom(&mut self) {
        self.zoom = 1.0;
        self.pending_zoom = None;
    }
    pub(crate) fn reset_position(&mut self) {
        self.offset = Default::default();
    }
}

impl Default for CanvasView {
    fn default() -> Self {
        Self {
            zoom: 1.0,
            offset: Default::default(),
            pending_zoom: None,
        }
    }
}

pub(super) fn handle_zoom(ui: &egui::Ui, view: &mut CanvasView, display: Vec2, avail: Rect) {
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

pub(super) fn handle_pan(view: &mut CanvasView, response: &Response) {
    if response.dragged() {
        view.offset += response.drag_delta();
    }
}

pub(super) fn get_lvgl_display_size() -> Vec2 {
    let w = unsafe { stratum_ui_common::stratum_ui_ffi::get_lvgl_display_width() as f32 };
    let h = unsafe { stratum_ui_common::stratum_ui_ffi::get_lvgl_display_height() as f32 };
    Vec2::new(w, h)
}

pub(super) fn full_uv() -> Rect {
    Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0))
}

pub(super) fn update_user_cursor_pos(
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

pub(super) fn compute_canvas_rect(view: &CanvasView, display: Vec2, avail: Rect) -> Rect {
    let scaled = display * view.zoom;
    let top_left = (avail.center() + view.offset - scaled * 0.5).round();
    Rect::from_min_size(top_left, scaled)
}
