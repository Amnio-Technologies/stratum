use crate::{
    flush_area_collector::{FrameRect, FLASH_DURATION},
    state::UiState,
};
use egui::Rect;

pub fn draw_flash_overlays(
    ui: &mut egui::Ui,
    ui_state: &UiState,
    view_rect: Rect,
    frame_rects: Vec<FrameRect>,
) {
    use egui::{Color32, Pos2, Rect, Stroke};
    use std::time::Instant;

    // nothing to do?
    if frame_rects.is_empty() {
        return;
    }

    let now = Instant::now();
    let painter = ui.painter();

    // Grab your CanvasView so you can transform coords
    let view = &ui_state.canvas_view;
    let canvas_tl = view_rect.min; // top-left on screen
    let zoom = view.zoom;

    for frame in frame_rects {
        let elapsed = now.saturating_duration_since(frame.timestamp);
        let alpha = (1.0 - elapsed.as_secs_f32() / FLASH_DURATION.as_secs_f32()).clamp(0.0, 1.0);
        let stroke_alpha = (alpha * 255.0).round() as u8;
        let fill_alpha = (alpha * 200.0).round() as u8;

        let stroke = Stroke::new(
            1.0,
            Color32::from_rgba_unmultiplied(0, 191, 255, stroke_alpha),
        );
        let fill = Color32::from_rgba_unmultiplied(0, 191, 255, fill_alpha);

        for lvgl_rect in &frame.rects {
            // Convert LVGL coords → screen coords
            let x0 = (canvas_tl.x + lvgl_rect.min.x * zoom).round();
            let y0 = (canvas_tl.y + lvgl_rect.min.y * zoom).round();
            let x1 = (canvas_tl.x + lvgl_rect.max.x * zoom).round();
            let y1 = (canvas_tl.y + lvgl_rect.max.y * zoom).round();

            let screen_rect = Rect::from_min_max(Pos2::new(x0, y0), Pos2::new(x1, y1));

            painter.rect_stroke(screen_rect, 0.0, stroke, egui::StrokeKind::Outside);
            painter.rect_filled(screen_rect, 0.0, fill);
        }
    }

    // keep repainting until all flashes expire
    ui.ctx().request_repaint();
}
