use std::{ffi::CStr, mem::MaybeUninit, process::Command};

use egui::{Color32, Id, Image, Pos2, Rect, Response, RichText, Sense, TextureHandle, Ui};
use egui_ltreeview::{Action, NodeBuilder, TreeView, TreeViewBuilder};
use stratum_ui_common::stratum_ui_ffi::{self, lv_obj_t, lvlens_meta_t};

use crate::{
    icon_manager::IconManager,
    lvgl_obj_tree::{TreeManager, TreeNode},
    state::UiState,
};

/// Convert a raw C string pointer to a Rust `String`. Returns `""` if null.
unsafe fn string_from_raw(ptr: *const ::std::os::raw::c_char) -> String {
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
    color: Color32,
) -> Response {
    // 1) Figure out how large the icon will be:
    let full_rect = ui.available_rect_before_wrap();
    let (_, icon_rect) = ui.spacing().icon_rectangles(full_rect);
    let icon_size = icon_rect.size();

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
    Image::new(tex).tint(color).paint_at(ui, place);

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
        const ICON_PX: u32 = 24;
        let braces = icon_manager
            .icon(include_bytes!("../../../../assets/icons/braces.svg"))
            .square(ICON_PX);
        let eye_fill = icon_manager
            .icon(include_bytes!("../../../../assets/icons/eye-fill.svg"))
            .square(ICON_PX);
        let eye_slash = icon_manager
            .icon(include_bytes!("../../../../assets/icons/eye-slash.svg"))
            .square(ICON_PX);

        NodeIcons {
            braces,
            eye_fill,
            eye_slash,
        }
    }
}

fn draw_node(
    ui: &mut Ui,
    icons: &NodeIcons,
    node: &TreeNode,
    shown: &mut bool,
    ancestor_hidden: bool,
) {
    let text_color = if !*shown || ancestor_hidden {
        ui.visuals().weak_text_color()
    } else {
        ui.visuals().text_color() // default
    };

    let braces_id = ui.make_persistent_id(("braces_icon", node.ptr));
    let _ = paint_icon_clickable(ui, &icons.braces, true, 3.0, braces_id, text_color);

    if node.class_name == "lv_label" {
        let raw_ptr = node.ptr as *const lv_obj_t;
        let label_text = unsafe { string_from_raw(stratum_ui_ffi::lvgl_label_text(raw_ptr)) };
        ui.label(
            RichText::new(format!("{}:", &node.class_name))
                .monospace()
                .color(text_color),
        );
        ui.label(RichText::new(format!("\"{label_text}\"")).color(text_color));
    } else {
        ui.label(
            RichText::new(&node.class_name)
                .monospace()
                .color(text_color),
        );
    }

    let eye_spacing = ui.spacing().item_spacing.x;
    let eye_resp = if *shown {
        let eye_id = ui.make_persistent_id(("eye_icon", node.ptr));
        paint_icon_clickable(ui, &icons.eye_fill, false, eye_spacing, eye_id, text_color)
    } else {
        let eye_id = ui.make_persistent_id(("eye_slash_icon", node.ptr));
        paint_icon_clickable(ui, &icons.eye_slash, false, eye_spacing, eye_id, text_color)
    };

    if eye_resp.clicked() {
        *shown = !*shown;
        unsafe {
            stratum_ui_ffi::lvgl_obj_set_shown(node.ptr as *mut lv_obj_t, *shown);
        }
    }
}

fn get_node_definition(ptr: usize) -> Result<(String, i32), ()> {
    unsafe {
        let mut meta_uninit = MaybeUninit::<lvlens_meta_t>::uninit();
        let meta_fetch_successful =
            stratum_ui_ffi::lvlens_get_metadata(ptr as *mut lv_obj_t, meta_uninit.as_mut_ptr());

        if meta_fetch_successful {
            let meta = meta_uninit.assume_init();
            let file = string_from_raw(meta.file);

            return Ok((file, meta.line));
        }
    }

    Err(())
}

fn go_to_node_definition(ptr: usize) -> Result<(), ()> {
    let (file, line) = get_node_definition(ptr)?;

    let code_cmd = if cfg!(windows) { "code.cmd" } else { "code" };

    if Command::new(code_cmd)
        .arg("--goto")
        .arg(format!("{}:{}", file, line))
        .spawn()
        .is_ok()
    {
        return Ok(());
    }

    Err(())
}

/// Recursively build the TreeView and simultaneously update `local_hidden`
/// to reflect any toggles the user made in this subtree.
fn add_node(
    builder: &mut TreeViewBuilder<'_, usize>,
    node: &TreeNode,
    icons: &NodeIcons,
    local_hidden: &mut Vec<usize>,
    hidden_before: bool,
    ancestor_hidden: bool,
) {
    let mut shown = !hidden_before;

    let this_node_hidden_now = !shown; // i.e. hidden_before XOR toggled?
    let effective_hidden_for_children = ancestor_hidden || this_node_hidden_now;

    let draw_node_fn = |ui: &mut Ui| draw_node(ui, icons, node, &mut shown, ancestor_hidden);
    let draw_context_menu_fn = |ui: &mut Ui| {
        if ui.button("Go to Definition").clicked() {
            go_to_node_definition(node.ptr);
            ui.close_menu();
        }
    };

    if node.children.is_empty() {
        let leaf = NodeBuilder::leaf(node.ptr)
            .label_ui(draw_node_fn)
            .context_menu(draw_context_menu_fn);
        builder.node(leaf);
    } else {
        let dir = NodeBuilder::dir(node.ptr)
            .label_ui(draw_node_fn)
            .context_menu(draw_context_menu_fn);
        builder.node(dir);

        for child in &node.children {
            // For each child, we need to know if it was hidden _before_ this frame.
            // That is, `hidden_before_child = local_hidden.contains(&child.ptr)`.
            let child_hidden_before = local_hidden.contains(&child.ptr);
            add_node(
                builder,
                child,
                icons,
                local_hidden,
                child_hidden_before,
                effective_hidden_for_children,
            );
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
                    false,
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
