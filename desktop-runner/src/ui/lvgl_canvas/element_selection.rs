use super::view::CanvasView;
use crate::lvgl_obj_tree::TreeManager;
use crate::lvgl_obj_tree::TreeNode;
use crate::state::UiState;
use egui::{Color32, Pos2, Rect, Stroke, Ui, Vec2};

pub fn draw_selected_node(ui: &mut Ui, ui_state: &mut UiState, view_rect: Rect) {
    // 1) Get selected pointer
    let selected_ptr = {
        let guard = ui_state.tree_manager.lock().unwrap();
        guard.tree_state.selected().first().cloned()
    };
    if selected_ptr.is_none() {
        return;
    }
    let selected_ptr = selected_ptr.unwrap();

    // 2) Update tree and find node
    if let Some(root) = TreeManager::update_and_take_root(&ui_state.tree_manager) {
        if let Some(node) = find_node_by_ptr(&root, selected_ptr) {
            let x = node.x as f32;
            let y = node.y as f32;
            let w = node.w as f32;
            let h = node.h as f32;
            let bounds = Rect::from_min_size(Pos2::new(x, y), Vec2::new(w, h));
            draw_selection_box(ui, &ui_state.canvas_view, view_rect, bounds);
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
