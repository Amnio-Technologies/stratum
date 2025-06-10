use crate::{
    hot_reload_manager::HotReloadManager, icon_manager::IconManager, lvgl_obj_tree::TreeManager,
    state::UiState, stratum_lvgl_ui::StratumLvglUI,
    ui::debug_panel::performance_page::LvglFpsLimit,
};
use eframe::{egui, CreationContext, Frame};
use egui::epaint::text::{FontInsert, InsertFontFamily};
use std::{
    path::PathBuf,
    sync::{atomic::Ordering, Arc, Mutex},
};
use stratum_ui_common::ui_logging::UiLogger;

pub struct StratumApp {
    ui_state: UiState,
    lvgl_ui: StratumLvglUI,
    last_fps: LvglFpsLimit,
}

impl<'ctx> StratumApp {
    pub fn new(cc: &'ctx CreationContext<'ctx>) -> Self {
        env_logger::init();

        let hot_reload_manager = Arc::new(Mutex::new(HotReloadManager::new(
            PathBuf::from("../stratum-ui/build/desktop/libstratum-ui.dll"),
            PathBuf::from("../stratum-ui/build.py"),
            vec![
                PathBuf::from("../stratum-ui/src"),
                PathBuf::from("../stratum-ui/include"),
            ],
        )));
        HotReloadManager::start(hot_reload_manager.clone());

        let tree_manager = TreeManager::new();
        let ui_logger: Arc<UiLogger> = UiLogger::new(10_000);
        let icon_manager = IconManager::new(cc.egui_ctx.clone(), "./.asset_cache");
        let ui_state = UiState::new(ui_logger, hot_reload_manager, tree_manager, icon_manager);
        let initial_fps = ui_state.lvgl_fps_limit.clone();

        let repaint_flash_enabled = ui_state.repaint_flash_active;

        let lvgl_ui =
            StratumLvglUI::new(&cc.egui_ctx, initial_fps.get_limit(), repaint_flash_enabled);

        Self::add_fonts(&cc.egui_ctx);

        Self {
            ui_state,
            lvgl_ui,
            last_fps: initial_fps,
        }
    }

    fn add_fonts(ctx: &egui::Context) {
        ctx.add_font(FontInsert::new(
            "atkinson",
            egui::FontData::from_static(include_bytes!(
                "../assets/fonts/AtkinsonHyperlegible-Regular.ttf"
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
            egui::FontData::from_static(include_bytes!(
                "../assets/fonts/JetBrainsMonoNL-Regular.ttf"
            )),
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
            TreeManager::bind_ffi_callback(self.ui_state.tree_manager.clone());
            self.lvgl_ui.reload_ui();
        }
    }
}

impl eframe::App for StratumApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut Frame) {
        self.hot_reload_check();

        if self.ui_state.lvgl_fps_limit != self.last_fps {
            self.lvgl_ui
                .set_fps_limit(self.ui_state.lvgl_fps_limit.get_limit());
            self.last_fps = self.ui_state.lvgl_fps_limit.clone();
        }

        if self.ui_state.repaint_flash_active != self.lvgl_ui.flush_collector.is_enabled() {
            self.lvgl_ui
                .flush_collector
                .set_enabled(self.ui_state.repaint_flash_active);
        }

        crate::ui::draw_ui(ctx, &mut self.ui_state, &mut self.lvgl_ui);
    }
}
