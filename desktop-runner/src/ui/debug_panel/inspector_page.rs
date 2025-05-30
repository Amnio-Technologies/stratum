use egui_ltreeview::{TreeView, TreeViewBuilder};
use stratum_ui_common::lvgl_obj_tree::TreeNode;

use crate::state::UiState;

fn draw_lvgl_obj_tree(ui: &mut egui::Ui, ui_state: &UiState) {
    if let Some(root) = ui_state.tree_manager.take_root() {
        ui.heading("LVGL Object Tree");
        TreeView::new(ui.make_persistent_id("lvgl-object-tree")).show(ui, |builder| {
            // A helper that recurses for each node:
            fn add_node(builder: &mut TreeViewBuilder<'_, usize>, node: &TreeNode) {
                // use `node.ptr` as an ID (usize impl Hash+Eq)
                if node.children.is_empty() {
                    // leaf
                    builder.leaf(node.ptr, &node.class_name);
                } else {
                    // directory
                    builder.dir(node.ptr, &node.class_name);
                    for child in &node.children {
                        add_node(builder, child);
                    }
                    builder.close_dir();
                }
            }

            // Kick off at the root:
            add_node(builder, &root);
        });
    }
}

fn draw_property_editor_ui(ui: &mut egui::Ui, ui_state: &UiState) {}

pub fn draw_inspector_debug_ui(ui: &mut egui::Ui, ui_state: &UiState) {
    ui.horizontal(|ui| {
        ui.selectable_label(true, "👆");
        ui.selectable_label(false, "💡");
    });
    ui.separator();
    draw_lvgl_obj_tree(ui, ui_state);
    ui.separator();
    draw_property_editor_ui(ui, ui_state);
}
