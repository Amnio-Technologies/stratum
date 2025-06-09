use crate::{
    lvgl_obj_tree::{TreeManager, TreeNode},
    state::UiState,
    stratum_lvgl_ui::{StratumLvglUI, RENDER_LOCK},
};
use egui::{
    Align2, CentralPanel, Color32, Context, Direction, Event, FontId, Frame, Layout, PointerButton,
    Pos2, Rect, Response, Sense, Stroke, TextureHandle, Ui, Vec2,
};
use stratum_ui_common::stratum_ui_ffi;

pub const ZOOM_MIN: f32 = 0.1;
pub const ZOOM_MAX: f32 = 200.0;

pub struct CanvasView {
    pub zoom: f32,
    pub offset: Vec2,
    pub pending_zoom: Option<f32>,
}

impl CanvasView {
    pub fn reset_zoom(&mut self) {
        self.zoom = 1.0;
        self.pending_zoom = None;
    }
    pub fn reset_position(&mut self) {
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
    let bg_resp = ui.interact(available_rect, ui.id().with("canvas_bg"), Sense::click());

    if bg_resp.clicked() {
        ui_state
            .tree_manager
            .lock()
            .unwrap()
            .tree_state
            .set_selected(vec![]);
    }

    //  Zoom / pan / draw the LVGL texture
    let view_rect = {
        let view = &mut ui_state.canvas_view;
        handle_zoom(ui, view, display_size, available_rect);

        let rect = compute_canvas_rect(view, display_size, available_rect);
        let response = ui.allocate_rect(rect, Sense::click_and_drag());

        update_pan(view, &response);
        draw_canvas(ui, tex, rect, &response);
        maybe_draw_pixel_grid(ui, view, rect, display_size);
        rect
    };

    let zoom = ui_state.canvas_view.zoom;
    update_user_cursor_pos(ui, ui_state, view_rect, display_size, zoom);

    {
        let view = &mut ui_state.canvas_view;
        let rect = compute_canvas_rect(view, display_size, available_rect);

        for event in ui.ctx().input(|i| i.events.clone()) {
            if let Event::PointerButton {
                pos,
                button: PointerButton::Primary,
                pressed: true,
                ..
            } = event
            {
                // Clicks inside the canvas when element selection is active select the corresponding lvgl element
                if rect.contains(pos) && ui_state.element_select_active {
                    if let Some((lvgl_x, lvgl_y)) = ui_state.cursor_pos {
                        TreeManager::request_obj_at_point(&ui_state.tree_manager, lvgl_x, lvgl_y);
                    }
                    ui_state.element_select_active = false;
                    let _guard = RENDER_LOCK.lock().unwrap();
                    unsafe {
                        stratum_ui_ffi::revert_clickability();
                    }
                }
            }
        }
    }

    // 4) Extract the selected_ptr *before* calling update_and_take_root
    let selected_ptr_opt = {
        let guard = ui_state.tree_manager.lock().unwrap();
        guard.tree_state.selected().first().cloned()
    };

    // 5) Now call update_and_take_root (which locks internally) and draw the highlight
    if let Some(selected_ptr) = selected_ptr_opt {
        if let Some(root) = TreeManager::update_and_take_root(&ui_state.tree_manager) {
            if let Some(selected_node) = find_node_by_ptr(&root, selected_ptr) {
                let (x, y, w, h) = (
                    selected_node.x as f32,
                    selected_node.y as f32,
                    selected_node.w as f32,
                    selected_node.h as f32,
                );

                let bounds = Rect::from_min_max(Pos2 { x, y }, Pos2 { x: x + w, y: y + h });
                draw_selection_box(ui, &ui_state.canvas_view, view_rect, bounds);
            }
        }
    }
}

fn draw_selection_box(ui: &mut Ui, view: &CanvasView, canvas_rect: Rect, lvgl_bounds: Rect) {
    // Unpack LVGL bounds
    let lx = lvgl_bounds.min.x;
    let ly = lvgl_bounds.min.y;
    let lw = lvgl_bounds.width() + 1.0;
    let lh = lvgl_bounds.height() + 1.0;

    // Convert LVGL‐space to screen‐space:
    let x_screen = (canvas_rect.min.x + lx * view.zoom).round();
    let y_screen = (canvas_rect.min.y + ly * view.zoom).round();
    let w_screen = (lw * view.zoom).round();
    let h_screen = (lh * view.zoom).round();

    let selection_rect = Rect::from_min_size(
        Pos2::new(x_screen, y_screen),
        egui::Vec2::new(w_screen, h_screen),
    );

    // Compute a stroke that is exactly 1 device pixel wide
    let pixels_per_point = ui.ctx().pixels_per_point();
    let stroke = Stroke {
        width: 1.0 / pixels_per_point,
        color: Color32::from_rgb(0, 191, 255),
    };

    // Draw the rectangle outline
    ui.painter()
        .rect_stroke(selection_rect, 0.0, stroke, egui::StrokeKind::Outside);
}

/// Recursively search `node` (and its children) for a `TreeNode` whose `.ptr` equals `target`.
fn find_node_by_ptr<'a>(node: &'a TreeNode, target: usize) -> Option<&'a TreeNode> {
    if node.ptr == target {
        return Some(node);
    }
    for child in &node.children {
        if let Some(found) = find_node_by_ptr(child, target) {
            return Some(found);
        }
    }
    None
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
