use crate::{
    hot_reload_manager::SharedHotReloadManager,
    state::{update_fps, UiState},
    stratum_lvgl_ui::StratumLvglUI,
};
use eframe::{egui, CreationContext, Frame};
use egui::epaint::text::{FontInsert, InsertFontFamily};
use std::sync::{atomic::Ordering, Arc};
use stratum_ui_common::{lvgl_obj_tree::TreeManager, ui_logging::UiLogger};

pub struct StratumApp {
    ui_state: UiState,
    lvgl_ui: StratumLvglUI,
    last_frame_start: std::time::Instant,
}

impl StratumApp {
    pub fn new(
        cc: &CreationContext<'_>,
        ui_logger: Arc<UiLogger>,
        hot_reload_manager: SharedHotReloadManager,
        tree_manager: Arc<TreeManager>,
    ) -> Self {
        let ui_state = UiState::new(cc, ui_logger, hot_reload_manager, tree_manager);
        let lvgl_ui = StratumLvglUI::new();

        Self::add_fonts(&cc.egui_ctx);

        Self {
            ui_state,
            lvgl_ui,
            last_frame_start: std::time::Instant::now(),
        }
    }

    fn add_fonts(ctx: &egui::Context) {
        ctx.add_font(FontInsert::new(
            "atkinson",
            egui::FontData::from_static(include_bytes!(
                "../fonts/AtkinsonHyperlegible-Regular.ttf"
            )),
            vec![
                InsertFontFamily {
                    family: egui::FontFamily::Proportional,
                    priority: egui::epaint::text::FontPriority::Highest,
                },
                InsertFontFamily {
                    family: egui::FontFamily::Monospace,
                    priority: egui::epaint::text::FontPriority::Lowest,
                },
            ],
        ));

        ctx.add_font(FontInsert::new(
            "jetbrains_mono",
            egui::FontData::from_static(include_bytes!("../fonts/JetBrainsMonoNL-Regular.ttf")),
            vec![
                InsertFontFamily {
                    family: egui::FontFamily::Proportional,
                    priority: egui::epaint::text::FontPriority::Lowest,
                },
                InsertFontFamily {
                    family: egui::FontFamily::Monospace,
                    priority: egui::epaint::text::FontPriority::Highest,
                },
            ],
        ));
    }

    fn hot_reload_check(&mut self) {
        if self
            .ui_state
            .hot_reload_manager
            .lock()
            .unwrap()
            .should_reload_ui
            .swap(false, Ordering::Relaxed)
        {
            self.ui_state.ui_logger.clone().bind_ffi_callback();
            self.ui_state.tree_manager.clone().bind_ffi_callback();
            self.lvgl_ui.reload_ui();
        }
    }
}

impl eframe::App for StratumApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut Frame) {
        self.hot_reload_check();

        crate::ui::draw_ui(ctx, &mut self.ui_state, &mut self.lvgl_ui);

        // Update FPS counter and loop
        update_fps(&mut self.ui_state, &self.last_frame_start);
        self.last_frame_start = std::time::Instant::now();
        ctx.request_repaint();
    }
}
