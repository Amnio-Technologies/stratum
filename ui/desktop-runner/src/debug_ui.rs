use crate::UiState;
use egui::Checkbox;

/// Creates the debugging UI inside a right-aligned panel.
pub fn create_debug_ui(ui: &mut egui::Ui, ui_state: &mut UiState) {
    ui.heading("amnIO UI Debugger");
    ui.label("🖥️ Debugging Panel for amnIO");

    ui.separator();

    ui.add(egui::Slider::new(&mut ui_state.slider_value, 0.0..=100.0).text("Debug Slider"));
    ui.label(format!("Slider Value: {:.2}", ui_state.slider_value));

    ui.separator();

    ui.add(Checkbox::new(&mut ui_state.enable_vsync, "Enable VSync"));
    ui.label("Toggle VSync for smoother rendering");

    ui.separator();

    ui.text_edit_multiline(&mut ui_state.debug_text);
    ui.label("Debug Console (Editable)");

    if ui.button("Quit Application").clicked() {
        ui_state.quit = true;
    }
}
