use amnio_firmware::modules::dummies::dummy_battery::DummyBatteryModule;
use egui::{Checkbox, ScrollArea};

use crate::state::UiState;

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

    // Pull in new logs from the UiLogger this frame
    let new_logs = ui_state.ui_logger.take_logs();
    ui_state.log_buffer.extend(new_logs);

    ui.horizontal(|ui| {
        ui.label("📝 Debug Logs:");
        let has_logs = !ui_state.log_buffer.is_empty();
        if ui
            .add_enabled(has_logs, egui::Button::new("🗑 Clear Logs"))
            .clicked()
        {
            ui_state.log_buffer.clear();
        }
    });

    let mut scroll_to_bottom = false;
    ScrollArea::vertical().max_height(200.0).show(ui, |ui| {
        let log_count = ui_state.log_buffer.len();
        unsafe {
            if log_count > LAST_LOG_COUNT {
                scroll_to_bottom = true;
            }
            LAST_LOG_COUNT = log_count;
        }
        for line in &ui_state.log_buffer {
            ui.monospace(line);
        }
        if scroll_to_bottom {
            ui.scroll_to_cursor(Some(egui::Align::BOTTOM));
        }
    });

    ui.separator();

    ui.heading("🔌 Connected Modules");
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

    // ─── 🔥 Hot Reload Debugger Panel ───────────────────────────────
    ui.heading("🔥 Hot Reload Debugger");

    // ui.add(Checkbox::new(
    //     &mut ui_state.auto_reload_enabled,
    //     "Auto Reload on .so Change",
    // ));

    // ui.horizontal(|ui| {
    //     if ui.button("⟲ Reload Plugin").clicked() {
    //         ui_state.request_reload = true;
    //     }

    //     if ui.button("⏪ Rollback Plugin").clicked() {
    //         ui_state.request_rollback = true;
    //     }
    // });

    // ui.label(format!("Plugin Path: {}", ui_state.plugin_path));
    // ui.label(format!("Status: {}", ui_state.plugin_status));
    // ui.label(format!("Last Reload: {}", ui_state.last_reload_time));
    // ui.label(format!("ABI Hash: {}", ui_state.plugin_abi_hash));

    // ui.separator();

    // ui.heading("📄 Reload Log");

    // ScrollArea::vertical().max_height(120.0).show(ui, |ui| {
    //     for entry in ui_state.reload_log.iter().rev().take(100).rev() {
    //         ui.monospace(entry);
    //     }
    // });
}
