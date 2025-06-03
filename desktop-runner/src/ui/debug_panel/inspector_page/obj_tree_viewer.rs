use std::ffi::CStr;

use egui::{Id, Image, Pos2, Rect, Response, RichText, Sense, TextureHandle, Ui};
use egui_ltreeview::{Action, NodeBuilder, TreeView, TreeViewBuilder};
use stratum_ui_common::stratum_ui_ffi::{self, lv_obj_t};

use crate::{
    icon_manager::IconManager,
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

    // 3) Make it clickable (no layout change).
    let response = ui.interact(place, unique_id, Sense::click());

    // 4) Actually paint the icon:
    Image::new(tex).tint(tint).paint_at(ui, place);

    // 5) Advance the cursor (so next widget lands correctly).
    if is_left {
        ui.add_space(icon_size.x + spacing);
    } else {
        ui.add_space(spacing + icon_size.x + spacing);
    }

    response
}

struct NodeIcons {
    braces: TextureHandle,
    eye_fill: TextureHandle,
    eye_slash: TextureHandle,
}

impl NodeIcons {
    fn load(icon_manager: &mut IconManager) -> Self {
        let braces = icon_manager
            .icon(include_bytes!("../../../../assets/icons/braces.svg"))
            .square(100);
        let eye_fill = icon_manager
            .icon(include_bytes!("../../../../assets/icons/eye-fill.svg"))
            .square(100);
        let eye_slash = icon_manager
            .icon(include_bytes!("../../../../assets/icons/eye-slash.svg"))
            .square(100);

        NodeIcons {
            braces,
            eye_fill,
            eye_slash,
        }
    }
}

fn draw_node(ui: &mut Ui, icons: &NodeIcons, node: &TreeNode, shown: &mut bool) {
    let braces_id = ui.make_persistent_id(("braces_icon", node.ptr));
    let _ = paint_icon_clickable(ui, &icons.braces, true, 3.0, braces_id);

    if node.class_name == "lv_label" {
        let raw_ptr = node.ptr as *const lv_obj_t;
        let label_text = unsafe { string_from_raw(stratum_ui_ffi::lvgl_label_text(raw_ptr)) };
        ui.label(RichText::new(format!("{}:", &node.class_name)).monospace());
        ui.label(RichText::new(format!("\"{label_text}\"")));
    } else {
        ui.label(RichText::new(&node.class_name).monospace());
    }

    let eye_spacing = ui.spacing().item_spacing.x;
    let eye_resp = if *shown {
        let eye_id = ui.make_persistent_id(("eye_icon", node.ptr));
        paint_icon_clickable(ui, &icons.eye_fill, false, eye_spacing, eye_id)
    } else {
        let eye_id = ui.make_persistent_id(("eye_slash_icon", node.ptr));
        paint_icon_clickable(ui, &icons.eye_slash, false, eye_spacing, eye_id)
    };

    if eye_resp.clicked() {
        *shown = !*shown;
    }
}

/// Recursively build the TreeView and simultaneously update `local_hidden`
/// to reflect any toggles the user made in this subtree.
fn add_node(
    builder: &mut TreeViewBuilder<'_, usize>,
    node: &TreeNode,
    icons: &NodeIcons,
    local_hidden: &mut Vec<usize>,
    hidden_before: bool,
) {
    // Determine if this node is currently "shown" according to `local_hidden`.
    let mut shown = !hidden_before;

    // Build either a leaf or a directory, passing `&mut shown` into the label UI.
    let label_fn = |ui: &mut Ui| draw_node(ui, icons, node, &mut shown);

    if node.children.is_empty() {
        let leaf = NodeBuilder::leaf(node.ptr).label_ui(label_fn);
        builder.node(leaf);
    } else {
        let dir = NodeBuilder::dir(node.ptr).label_ui(label_fn);
        builder.node(dir);

        for child in &node.children {
            // For each child, we need to know if it was hidden _before_ this frame.
            // That is, `hidden_before_child = local_hidden.contains(&child.ptr)`.
            let child_hidden_before = local_hidden.contains(&child.ptr);
            add_node(builder, child, icons, local_hidden, child_hidden_before);
        }

        builder.close_dir();
    }

    // After drawing the label (and possibly clicking the eye icon), `shown` tells us
    // if the node is visible _now_. Compare with `hidden_before` to know if it changed.

    if shown && hidden_before {
        // Node was unhidden this frame. Remove it from local_hidden if present.
        if let Some(pos) = local_hidden.iter().position(|x| *x == node.ptr) {
            local_hidden.remove(pos);
        }
    } else if !shown && !hidden_before {
        // Node was hidden this frame. Add it to local_hidden if not already there.
        if !local_hidden.contains(&node.ptr) {
            local_hidden.push(node.ptr);
        }
    }
}

pub fn draw(ui: &mut egui::Ui, ui_state: &mut UiState) {
    // 1) Lock once and extract both tree_state and hidden_elements
    let (mut tree_state, mut local_hidden) = {
        let mgr = ui_state.tree_manager.lock().unwrap();
        (mgr.tree_state.clone(), mgr.hidden_elements.clone())
    };

    // 2) Ask TreeManager to produce a new root for this frame (also needs a lock).
    let root = TreeManager::update_and_take_root(&ui_state.tree_manager);

    if let Some(root) = root {
        ui.style_mut().interaction.selectable_labels = false;

        let icons = NodeIcons::load(&mut ui_state.icon_manager);

        let tree_id = ui.make_persistent_id("lvgl-object-tree");
        let (_resp, actions) = TreeView::new(tree_id)
            .allow_multi_selection(false)
            .override_indent(Some(12.0))
            .show_state(ui, &mut tree_state, |builder| {
                // Kick off recursion with root. We pass in:
                //   - a mutable reference to local_hidden,
                //   - whether root was hidden_before (local_hidden.contains(&root.ptr)).
                let root_hidden_before = local_hidden.contains(&root.ptr);
                add_node(
                    builder,
                    &root,
                    &icons,
                    &mut local_hidden,
                    root_hidden_before,
                );
            });

        for action in actions {
            if let Action::SetSelected(v) = action {
                dbg!(v);
            }
        }

        // Now that `local_hidden` reflects this frame’s toggles, write it back under a short lock:
        {
            let mut mgr = ui_state.tree_manager.lock().unwrap();
            mgr.tree_state = tree_state;
            mgr.hidden_elements = local_hidden;
        }
    }
}
