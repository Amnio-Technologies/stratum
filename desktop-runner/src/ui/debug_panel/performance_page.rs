use std::fmt::Display;

use egui::ComboBox;

use crate::state::UiState;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum LvglFpsLimit {
    Preset(u32),
    Custom(u32),
}

impl LvglFpsLimit {
    pub fn get_limit(&self) -> Option<u32> {
        match self {
            Self::Preset(v) | Self::Custom(v) => Some(*v),
        }
    }
}

impl Display for LvglFpsLimit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LvglFpsLimit::Preset(v) => write!(f, "{} FPS", v),
            LvglFpsLimit::Custom(_) => write!(f, "Custom"),
        }
    }
}

pub(super) fn draw(ui: &mut egui::Ui, ui_state: &mut UiState) {
    ui.heading("LVGL UI Framerate");

    const PRESETS: [u32; 7] = [1, 5, 10, 15, 30, 60, 120];

    let selected_limit = &mut ui_state.lvgl_fps_limit;

    ComboBox::from_label("LVGL FPS Limit")
        .selected_text(selected_limit.to_string())
        .show_ui(ui, |ui| {
            PRESETS.map(|preset| {
                let preset_value = LvglFpsLimit::Preset(preset);
                if ui
                    .selectable_label(*selected_limit == preset_value, format!("{preset} FPS"))
                    .clicked()
                {
                    *selected_limit = preset_value;
                }
            });

            if ui
                .selectable_label(
                    matches!(*selected_limit, LvglFpsLimit::Custom(_)),
                    format!("Custom"),
                )
                .clicked()
            {
                *selected_limit = LvglFpsLimit::Custom(30);
            }
        });

    if let LvglFpsLimit::Custom(ref mut custom_fps) = ui_state.lvgl_fps_limit {
        ui.horizontal(|ui| {
            ui.label("Custom FPS:");
            ui.add(
                egui::DragValue::new(custom_fps)
                    .range(1..=240)
                    .suffix(" FPS"),
            );
        });
    }
}
