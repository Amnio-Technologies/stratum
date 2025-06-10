use crate::state::UiState;
use egui::Id;
use egui::ScrollArea;

pub fn draw(ui: &mut egui::Ui, ui_state: &mut UiState) {
    let logger = &ui_state.ui_logger;
    let all_logs = logger.all_logs();

    ui.horizontal(|ui| {
        ui.label(format!("📝 Debug Logs ({}):", all_logs.len()));
        let has_logs = !all_logs.is_empty();
        if ui
            .add_enabled(has_logs, egui::Button::new("🗑 Clear Logs"))
            .clicked()
        {
            logger.take_logs();
        }
    });

    let scroll_id = Id::new("debug_log_scroll");

    ScrollArea::vertical()
        .id_salt(scroll_id)
        .auto_shrink(false)
        .stick_to_bottom(true)
        .show(ui, |ui| {
            for line in &all_logs {
                ui.monospace(line);
            }
        });
}
