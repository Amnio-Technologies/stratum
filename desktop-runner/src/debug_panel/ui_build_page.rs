use egui::{Checkbox, Id, ScrollArea};
use stratum_firmware_common::modules::dummies::dummy_battery::DummyBatteryModule;

use crate::state::UiState;

pub fn draw_uibuild_debug_ui(ui: &mut egui::Ui, ui_state: &mut UiState) {
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

    let scroll_id = Id::new("debug_log_scroll");

    ScrollArea::vertical()
        .id_salt(scroll_id)
        .stick_to_bottom(true)
        .max_height(200.0)
        .show(ui, |ui| {
            for line in &ui_state.log_buffer {
                ui.monospace(line);
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

    let manager = &mut ui_state.hot_reload_manager.lock().unwrap();

    ui.group(|ui| {
        // Row: Auto reload toggle + max builds spinner
        ui.horizontal(|ui| {
            ui.checkbox(&mut manager.auto_reload, "Auto Reload on File Change");

            ui.label("Max Builds to Keep:");
            ui.add(
                egui::DragValue::new(&mut manager.max_builds_to_keep)
                    .range(1..=20)
                    .speed(1),
            );
        });

        ui.separator();

        // Active plugin info
        ui.group(|ui| {
            ui.label(egui::RichText::new("📦 Active Plugin").strong());

            ui.label(format!("Status:        {}", manager.status));
            ui.label(format!("Last Reloaded: {}", manager.last_reload_timestamp));
            ui.label(format!("ABI Hash:      {}", manager.current_abi_hash));
        });

        ui.separator();

        ui.label(egui::RichText::new("📃 Available Builds").strong());

        egui::ComboBox::from_id_salt("build_selector")
            .width(ui.available_width())
            .selected_text(manager.selected_build_display())
            .show_ui(ui, |cb| {
                for build in &manager.sorted_builds() {
                    let label = if build.is_active {
                        format!("{} [ACTIVE]", build.filename())
                    } else {
                        build.filename().clone()
                    };

                    cb.selectable_value(
                        &mut ui_state.selected_build,
                        Some(build.clone().path),
                        label,
                    );
                }
            });

        if ui.button("Load Selected").clicked() {
            if let Some(build_path) = &ui_state.selected_build {
                manager.load_plugin(build_path.as_path());
            }
        }

        ui.separator();

        ui.label(egui::RichText::new("📜 Reload Log").strong());

        fn sanitize(entry: &str) -> String {
            entry
                .chars()
                // strip out FE0F (emoji VS-16) and FFFD (� replacement char)
                .filter(|&c| c != '\u{FE0F}' && c != '\u{FFFD}')
                .collect()
        }

        // give the reload‐log scroll area a stable ID:
        let reload_scroll_id = Id::new("reload_log_scroll");

        ScrollArea::vertical()
            .id_salt(reload_scroll_id)
            .stick_to_bottom(true)
            .max_height(200.0)
            .show(ui, |ui| {
                for entry in manager.reload_log.iter().rev().take(100).rev() {
                    let clean = sanitize(entry);
                    ui.label(egui::RichText::new(clean).monospace());
                }
            });
    });
}
