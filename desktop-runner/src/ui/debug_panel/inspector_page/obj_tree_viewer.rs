use std::ffi::CStr;

use egui::{Id, Image, Pos2, Rect, Response, RichText, Sense, TextureHandle, Ui};
use egui_ltreeview::{Action, NodeBuilder, TreeView, TreeViewBuilder};
use stratum_ui_common::stratum_ui_ffi::{self, lv_obj_t};

use crate::{
    lvgl_obj_tree::{TreeManager, TreeNode},
    state::UiState,
};

/// Convert a raw C string pointer to a Rust `String`. Returns `""` if null.
unsafe fn string_from_raw(ptr: *mut ::std::os::raw::c_char) -> String {
    if ptr.is_null() {
        String::new()
    } else {
        CStr::from_ptr(ptr).to_string_lossy().into_owned()
    }
}

fn paint_icon_clickable(
    ui: &mut Ui,
    tex: &egui::TextureHandle,
    is_left: bool,
    spacing: f32,
    unique_id: Id,
) -> Response {
    // 1) Figure out how large the icon will be:
    let full_rect = ui.available_rect_before_wrap();
    let (_, icon_rect) = ui.spacing().icon_rectangles(full_rect);
    let icon_size = icon_rect.size();
    let tint = ui.visuals().widgets.noninteractive.fg_stroke.color;

    // 2) Compute “place” based on left vs. right:
    let place = if is_left {
        Rect::from_min_size(
            Pos2 {
                x: ui.cursor().min.x,
                y: icon_rect.min.y,
            },
            icon_size,
        )
    } else {
        let leftover = ui.available_width() - icon_size.x - spacing;
        Rect::from_min_size(
            Pos2 {
                x: ui.cursor().min.x + leftover,
                y: icon_rect.min.y,
            },
            icon_size,
        )
    };

    // 3) Use `interact` (no layout impact) to make it clickable:
    let response = ui.interact(place, unique_id, Sense::click());

    // 4) Paint the icon at exactly the same rect:
    Image::new(tex).tint(tint).paint_at(ui, place);

    // 5) Advance the cursor so that subsequent UI elements lay out correctly:
    if is_left {
        ui.add_space(icon_size.x + spacing);
    } else {
        // match the old “right” version’s spacing so we skip over the icon + padding:
        ui.add_space(spacing + icon_size.x + spacing);
    }

    response
}

struct LabelIcons {
    braces: TextureHandle,
    eye_fill: TextureHandle,
    eye_slash: TextureHandle,
}

fn draw_node_label(ui: &mut Ui, icons: &LabelIcons, node: &TreeNode, shown: &mut bool) {
    let braces_id = ui.make_persistent_id(("braces_icon", node.ptr));
    let resp_braces = paint_icon_clickable(ui, &icons.braces, true, 3.0, braces_id);

    if node.class_name == "lv_label" {
        let raw_ptr = node.ptr as *const lv_obj_t;
        let label_text = unsafe { string_from_raw(stratum_ui_ffi::lvgl_label_text(raw_ptr)) };
        ui.label(RichText::new(format!("{}:", &node.class_name)).monospace());
        ui.label(RichText::new(format!("\"{label_text}\"")));
    } else {
        ui.label(RichText::new(&node.class_name).monospace());
    }

    let eye_spacing = ui.spacing().item_spacing.x;
    // paint_icon_clickable(ui, &eye_fill_tex, /* is_left = */ false, eye_spacing);
    let eye_resp = if *shown {
        let eye_id = ui.make_persistent_id(("eye_icon", node.ptr));
        paint_icon_clickable(ui, &icons.eye_fill, false, eye_spacing, eye_id)
    } else {
        let eye_id = ui.make_persistent_id(("eye_slash_icon", node.ptr));
        paint_icon_clickable(ui, &icons.eye_slash, false, eye_spacing, eye_id)
    };
}

fn add_node(builder: &mut TreeViewBuilder<'_, usize>, node: &TreeNode, ui_state: &mut UiState) {
    let braces_tex = ui_state
        .icon_manager
        .icon(include_bytes!("../../../../assets/icons/braces.svg"))
        .square(100);
    let eye_fill_tex = ui_state
        .icon_manager
        .icon(include_bytes!("../../../../assets/icons/eye-fill.svg"))
        .square(100);
    let eye_slash_tex = ui_state
        .icon_manager
        .icon(include_bytes!("../../../../assets/icons/eye-slash.svg"))
        .square(100);

    let icons = LabelIcons {
        braces: braces_tex,
        eye_fill: eye_fill_tex,
        eye_slash: eye_slash_tex,
    };

    let shown_before = {
        let mgr = ui_state.tree_manager.lock().unwrap();
        mgr.hidden_elements.contains(&node.ptr)
    };

    let mut shown = shown_before;

    if node.children.is_empty() {
        // leaf
        let leaf = NodeBuilder::leaf(node.ptr)
            .label_ui(|ui| draw_node_label(ui, &icons, node, &mut shown));
        builder.node(leaf);
    } else {
        // directory
        let dir =
            NodeBuilder::dir(node.ptr).label_ui(|ui| draw_node_label(ui, &icons, node, &mut shown));

        builder.node(dir);
        for child in &node.children {
            add_node(builder, child, ui_state);
        }

        builder.close_dir();
    }

    if shown && !shown_before {
        let mut mgr = ui_state.tree_manager.lock().unwrap();
        // Remove this node.ptr from hidden_elements (if present)
        if let Some(pos) = mgr.hidden_elements.iter().position(|x| *x == node.ptr) {
            mgr.hidden_elements.remove(pos);
        }
    }
    if !shown && shown_before {
        let mut mgr = ui_state.tree_manager.lock().unwrap();
        // Push node.ptr onto hidden_elements
        mgr.hidden_elements.push(node.ptr);
    }
}

pub fn draw(ui: &mut egui::Ui, ui_state: &mut UiState) {
    let shared_mgr = ui_state.tree_manager.clone();
    let root = TreeManager::update_and_take_root(&shared_mgr);

    if let Some(root) = root {
        ui.style_mut().interaction.selectable_labels = false;

        let id: Id = ui.make_persistent_id("lvgl-object-tree");
        let state = &mut shared_mgr.lock().unwrap().tree_state;

        let (_resp, actions) = TreeView::new(id)
            .allow_multi_selection(false)
            .override_indent(Some(12.0))
            .show_state(ui, state, |builder| {
                // Kick off at the root:
                add_node(builder, &root, ui_state);
            });

        for action in actions {
            if let Action::SetSelected(v) = action {
                dbg!(v);
            }
        }
    }
}
