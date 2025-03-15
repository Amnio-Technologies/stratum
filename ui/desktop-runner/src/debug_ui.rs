use amnio_common::ui_logging::UI_LOGS;
use amnio_firmware::modules::dummies::dummy_battery::DummyBatteryModule;
use egui::{Checkbox, ScrollArea};

use crate::UiState;

pub const DEBUG_UI_WIDTH: u32 = 300;

/// Tracks last log count to determine when to auto-scroll
static mut LAST_LOG_COUNT: usize = 0;

/// Creates the debugging UI inside a right-aligned panel.
pub fn create_debug_ui(ui: &mut egui::Ui, ui_state: &mut UiState) {
    ui.heading("amnIO UI Debugger");

    ui.separator();
    ui.label(format!("FPS: {:.2}", ui_state.fps));

    ui.separator();
    ui.add(Checkbox::new(&mut ui_state.enable_vsync, "Enable VSync"));
    ui.label("Toggle VSync for smoother rendering");

    ui.separator();

    ui.horizontal(|ui| {
        ui.label("📝 Debug Logs:");

        let has_logs = UI_LOGS.try_lock().map_or(false, |logs| !logs.is_empty());

        let clear_button = ui.add_enabled(has_logs, egui::Button::new("🗑 Clear Logs"));
        if clear_button.clicked() {
            if let Ok(mut logs) = UI_LOGS.try_lock() {
                logs.clear();
            }
        }
    });

    let mut scroll_to_bottom = false;

    ScrollArea::vertical().max_height(200.0).show(ui, |ui| {
        if let Ok(logs) = UI_LOGS.try_lock() {
            let log_count = logs.len();

            unsafe {
                if log_count > LAST_LOG_COUNT {
                    scroll_to_bottom = true;
                }
                LAST_LOG_COUNT = log_count;
            }

            for log in logs.iter().rev().take(100).rev() {
                ui.monospace(log);
            }

            if scroll_to_bottom {
                ui.scroll_to_cursor(Some(egui::Align::BOTTOM));
            }
        } else {
            ui.label("⚠️ Logs unavailable (locked)");
        }
    });

    ui.separator();

    ui.heading("🔌 Connected Modules");

    // Display connected modules
    let connected_modules = ui_state.module_manager.list_modules();
    if connected_modules.is_empty() {
        ui.label("No modules connected.");
    } else {
        for module_metadata in connected_modules.iter() {
            ui.horizontal(|ui| {
                ui.label(format!("Module ID: {}", module_metadata.id));
                if ui.button("🗑 Remove").clicked() {
                    ui_state.module_manager.remove_module(module_metadata.id);
                }
            });
        }
    }

    ui.separator();

    if ui.button("➕ Add Battery Module").clicked() {
        let dummy_module = DummyBatteryModule::new(ui_state.module_manager.generate_unique_id());

        ui_state
            .module_manager
            .register_module(dummy_module, ui_state.system_controller.clone());
    }

    ui.separator();

    // Quit button
    if ui.button("Quit Application").clicked() {
        ui_state.quit = true;
    }
}
