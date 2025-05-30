use crate::state::UiState;

pub fn draw(ctx: &egui::Context, ui_state: &mut UiState) {
    egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
        egui::menu::bar(ui, |ui| {
            ui.menu_button("File", |ui| {
                if ui.button("Save").clicked() {
                    //functionality
                }
                if ui.button("Quit").clicked() {
                    std::process::exit(0);
                }
            });

            ui.menu_button("Edit", |ui| {
                if ui.button("Cut").clicked() {
                    //functionality
                }
                if ui.button("Copy").clicked() {
                    //functionality
                }
                if ui.button("Paste").clicked() {
                    //funtionality
                }
            })
        });
    });
}
