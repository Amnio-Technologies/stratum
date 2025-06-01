use std::ffi::CStr;

use egui::{Image, Pos2, Rect, RichText, Ui};
use egui_ltreeview::{NodeBuilder, TreeView, TreeViewBuilder};
use stratum_ui_common::{
    lvgl_obj_tree::TreeNode,
    stratum_ui_ffi::{self, lv_obj_t},
};
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

use crate::state::UiState;

use super::pages::DebugSidebarPages;

fn draw_lvgl_obj_tree(ui: &mut egui::Ui, ui_state: &mut UiState) {
    if let Some(root) = ui_state.tree_manager.take_root() {
        ui.style_mut().interaction.selectable_labels = false;

        TreeView::new(ui.make_persistent_id("lvgl-object-tree")).show(ui, |builder| {
            // A helper that recurses for each node:
            fn add_node(
                builder: &mut TreeViewBuilder<'_, usize>,
                node: &TreeNode,
                ui_state: &mut UiState,
            ) {
                let eye_fill = ui_state
                    .icon_manager
                    .icon(include_bytes!("../../../assets/icons/eye-fill.svg"))
                    .square(100);

                let braces = ui_state
                    .icon_manager
                    .icon(include_bytes!("../../../assets/icons/braces.svg"))
                    .square(100);

                let draw_label = |ui: &mut Ui| {
                    unsafe fn c_char_ptr_to_string(ptr: *mut ::std::os::raw::c_char) -> String {
                        if ptr.is_null() {
                            return String::new();
                        }
                        CStr::from_ptr(ptr).to_string_lossy().into_owned()
                    }

                    if node.class_name == "lv_label" {
                        let label_text = unsafe {
                            let obj_ptr = node.ptr as *const lv_obj_t;
                            c_char_ptr_to_string(stratum_ui_ffi::lvgl_label_text(obj_ptr))
                        };
                        ui.label(RichText::new(format!("{}:", &node.class_name)).monospace());

                        ui.label(RichText::new(format!("\"{label_text}\"")));
                    } else {
                        ui.label(RichText::new(&node.class_name).monospace());
                    }

                    let (_, big_rect) = ui
                        .spacing()
                        .icon_rectangles(ui.available_rect_before_wrap());

                    let spacing = ui.spacing().item_spacing.x;
                    let leftover = ui.available_width() - big_rect.width() - spacing;

                    let place = Rect::from_min_size(
                        Pos2 {
                            x: ui.cursor().min.x + leftover,
                            y: big_rect.min.y,
                        },
                        big_rect.size(),
                    );

                    ui.add_space(spacing + big_rect.width() + spacing);

                    let img = Image::new(&eye_fill)
                        .tint(ui.visuals().widgets.noninteractive.fg_stroke.color);
                    img.paint_at(ui, place);
                };

                let draw_icon = |ui: &mut Ui| {
                    let img = Image::new(&braces)
                        .tint(ui.visuals().widgets.noninteractive.fg_stroke.color);
                    img.paint_at(ui, ui.max_rect());
                };

                // use `node.ptr` as an ID (usize impl Hash+Eq)
                if node.children.is_empty() {
                    // leaf
                    let leaf = NodeBuilder::leaf(node.ptr)
                        .label_ui(draw_label)
                        .icon(draw_icon);

                    builder.node(leaf);
                } else {
                    // directory
                    let dir = NodeBuilder::dir(node.ptr)
                        .label_ui(draw_label)
                        .icon(draw_icon);

                    builder.node(dir);
                    for child in &node.children {
                        add_node(builder, child, ui_state);
                    }
                    builder.close_dir();
                }
            }

            // Kick off at the root:
            add_node(builder, &root, ui_state);
        });
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
