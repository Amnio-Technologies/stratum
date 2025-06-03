use std::ffi::CStr;

use egui::{Id, Image, Pos2, Rect, Response, RichText, Sense, Ui};
use egui_ltreeview::{Action, NodeBuilder, TreeView, TreeViewBuilder};
use stratum_ui_common::stratum_ui_ffi::{self, lv_obj_t};
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

use crate::{
    lvgl_obj_tree::{TreeManager, TreeNode},
    state::UiState,
};

use super::pages::DebugSidebarPages;

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

fn draw_node_label(ui: &mut Ui, node: &TreeNode, ui_state: &mut UiState) {
    let braces_tex = ui_state
        .icon_manager
        .icon(include_bytes!("../../../assets/icons/braces.svg"))
        .square(100);
    let eye_fill_tex = ui_state
        .icon_manager
        .icon(include_bytes!("../../../assets/icons/eye-fill.svg"))
        .square(100);

    let braces_id = ui.make_persistent_id(("braces_icon", node.ptr));
    let resp_braces = paint_icon_clickable(ui, &braces_tex, true, 3.0, braces_id);

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
    let eye_id = ui.make_persistent_id(("eye_icon", node.ptr));
    let resp_eye = paint_icon_clickable(ui, &eye_fill_tex, false, eye_spacing, eye_id);
}

fn add_node(builder: &mut TreeViewBuilder<'_, usize>, node: &TreeNode, ui_state: &mut UiState) {
    // use `node.ptr` as an ID (usize impl Hash+Eq)
    if node.children.is_empty() {
        // leaf
        let leaf = NodeBuilder::leaf(node.ptr).label_ui(|ui| draw_node_label(ui, node, ui_state));
        builder.node(leaf);
    } else {
        // directory
        let dir = NodeBuilder::dir(node.ptr).label_ui(|ui| draw_node_label(ui, node, ui_state));

        builder.node(dir);
        for child in &node.children {
            add_node(builder, child, ui_state);
        }

        builder.close_dir();
    }
}

fn draw_lvgl_obj_tree(ui: &mut egui::Ui, ui_state: &mut UiState) {
    let shared_mgr = ui_state.tree_manager.clone();
    let root = TreeManager::update_and_take_root(&shared_mgr);

    if let Some(root) = root {
        ui.style_mut().interaction.selectable_labels = false;

        let id = ui.make_persistent_id("lvgl-object-tree");
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

#[derive(Debug, Clone, Copy, PartialEq, EnumIter)]
pub enum PropertyEditorTabs {
    BaseProperties,
    StyleProperties,
    Events,
}

fn draw_base_properties_editor_tab(ui: &mut egui::Ui, ui_state: &UiState) {}

fn draw_style_properties_editor_tab(ui: &mut egui::Ui, ui_state: &UiState) {}

fn draw_events_editor_tab(ui: &mut egui::Ui, ui_state: &UiState) {}

impl PropertyEditorTabs {
    pub fn as_str(&self) -> &'static str {
        match self {
            PropertyEditorTabs::BaseProperties => "Base",
            PropertyEditorTabs::StyleProperties => "Style",
            PropertyEditorTabs::Events => "Events",
        }
    }

    pub fn draw_debug_page(&self, ui: &mut egui::Ui, ui_state: &mut UiState) {
        match self {
            PropertyEditorTabs::BaseProperties => draw_base_properties_editor_tab(ui, ui_state),
            PropertyEditorTabs::StyleProperties => draw_style_properties_editor_tab(ui, ui_state),
            PropertyEditorTabs::Events => draw_events_editor_tab(ui, ui_state),
        }
    }
}

impl Default for PropertyEditorTabs {
    fn default() -> Self {
        Self::BaseProperties
    }
}

fn draw_property_editor_ui(ui: &mut egui::Ui, ui_state: &mut UiState) {
    if let DebugSidebarPages::Inspector(selected_tab) = ui_state.selected_debug_page {
        // 1) Collect all the tab variants
        let tabs: Vec<_> = PropertyEditorTabs::iter().collect();
        let count = tabs.len() as f32;

        // 2) Compute how wide each tab should be:
        let spacing = ui.spacing().item_spacing.x;
        let total_spacing = spacing * (count - 1.0);
        let avail = ui.available_width();
        let tab_width = (avail - total_spacing) / count;

        // 3) Lay them out in a horizontal row, each sized to `tab_width`
        ui.horizontal_top(|ui| {
            for &tab in &tabs {
                let is_selected = tab == selected_tab;
                let lbl = egui::SelectableLabel::new(is_selected, tab.as_str());
                // height = 0.0 → use default interact height
                let resp = ui.add_sized([tab_width, 0.0], lbl);
                if resp.clicked() {
                    ui_state.selected_debug_page = DebugSidebarPages::Inspector(tab);
                }
            }
        });

        selected_tab.draw_debug_page(ui, ui_state);
    }
}

pub fn draw_inspector_debug_ui(ui: &mut egui::Ui, ui_state: &mut UiState) {
    ui.horizontal(|ui| {
        ui.selectable_label(true, "👆");
        ui.selectable_label(false, "💡");
    });
    ui.separator();
    draw_lvgl_obj_tree(ui, ui_state);
    ui.separator();
    draw_property_editor_ui(ui, ui_state);
}
