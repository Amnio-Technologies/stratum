use super::element_selection::draw_selected_node;
use super::interaction::{
    clear_element_selection_on_background_click, handle_canvas_click_selection,
};
use super::repaint_flash::draw_flash_overlays;
use super::view::{
    compute_canvas_rect, full_uv, get_lvgl_display_size, handle_pan, handle_zoom,
    update_user_cursor_pos, CanvasView,
};
use crate::{flush_area_collector::FrameRect, state::UiState};
use egui::{Align2, Color32, FontId, Pos2, Rect, Response, Sense, Stroke, TextureHandle, Vec2};

fn paint_lvgl_ui_framebuffer(
    ui: &mut egui::Ui,
    tex: Option<TextureHandle>,
    rect: Rect,
    response: &Response,
) {
    if let (Some(tex), true) = (&tex, response.hovered() || response.dragged()) {
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

fn render_canvas(
    ui: &mut egui::Ui,
    ui_state: &mut UiState,
    tex: Option<TextureHandle>,
    display: Vec2,
    avail: Rect,
) -> Rect {
    let view = &mut ui_state.canvas_view;
    handle_zoom(ui, view, display, avail);
    let rect = compute_canvas_rect(view, display, avail);
    let resp = ui.allocate_rect(rect, Sense::click_and_drag());
    handle_pan(view, &resp);
    paint_lvgl_ui_framebuffer(ui, tex, rect, &resp);
    maybe_draw_pixel_grid(ui, view, rect, display);
    rect
}

pub fn draw_lvgl_canvas(
    ui: &mut egui::Ui,
    ui_state: &mut UiState,
    tex: Option<TextureHandle>,
    frame_rects: Vec<FrameRect>,
) {
    // Existing canvas texture drawing
    let display_size = get_lvgl_display_size();
    let available_rect = ui.available_rect_before_wrap();

    clear_element_selection_on_background_click(ui, ui_state, available_rect);

    // Zoom, pan, and draw the LVGL texture
    let view_rect = render_canvas(ui, ui_state, tex, display_size, available_rect);

    // Overlay flush-frame highlights with fade-out fill
    draw_flash_overlays(ui, ui_state, view_rect, frame_rects);

    // Update cursor and draw selection as before
    update_user_cursor_pos(
        ui,
        ui_state,
        view_rect,
        display_size,
        ui_state.canvas_view.zoom,
    );

    handle_canvas_click_selection(ui, ui_state, view_rect);

    // Draw selected node highlight
    draw_selected_node(ui, ui_state, view_rect);
}
