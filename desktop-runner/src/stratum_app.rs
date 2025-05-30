use crate::{
    debug_panel,
    hot_reload_manager::SharedHotReloadManager,
    state::{update_fps, UiState},
    stratum_lvgl_ui::StratumLvglUI,
};
use eframe::{egui, CreationContext, Frame};
use egui::{
    epaint::text::{FontInsert, InsertFontFamily},
    Direction, Layout, ScrollArea, TextureHandle,
};
use std::sync::{atomic::Ordering, Arc};
use stratum_ui_common::{lvgl_obj_tree::TreeManager, ui_logging::UiLogger};

pub struct StratumApp {
    ui_state: UiState,
    lvgl_ui: StratumLvglUI,
    lvgl_tex: Option<TextureHandle>,
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
            lvgl_tex: None,
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
        // Hot-reload check
        self.hot_reload_check();

        // Generate the latest LVGL texture
        self.lvgl_tex = self.lvgl_ui.update(ctx).cloned();

        draw_debug_panel(ctx, &mut self.ui_state);

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

        egui::CentralPanel::default()
            .frame(
                egui::Frame::central_panel(&ctx.style()).fill(egui::Color32::from_rgb(20, 20, 20)),
            )
            .show(ctx, |ui| {
                ui.with_layout(
                    Layout::centered_and_justified(Direction::LeftToRight),
                    |ui| {
                        draw_lvgl_canvas(ui, self.lvgl_tex.as_ref());
                    },
                );
            });

        egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(format!("FPS: {:.2}", self.ui_state.fps));
                ui.separator();
                ui.label(format!("Zoom: {:.0}%", self.ui_state.zoom * 100.0));
            });
        });

        // Update FPS counter and loop
        update_fps(&mut self.ui_state, &self.last_frame_start);
        self.last_frame_start = std::time::Instant::now();
        ctx.request_repaint();
    }
}

// ---------- helpers ----------

fn draw_lvgl_canvas(ui: &mut egui::Ui, tex: Option<&TextureHandle>) {
    let width = unsafe { stratum_ui_common::stratum_ui_ffi::get_lvgl_display_width() as f32 };
    let height = ui.available_height();

    // Allocate exactly the display size and center the image
    ui.allocate_ui_with_layout(
        egui::vec2(width, height),
        Layout::centered_and_justified(Direction::LeftToRight),
        |ui| {
            if let Some(t) = tex {
                ui.image(t);
            } else {
                ui.label("No LVGL texture");
            }
        },
    );
}

fn draw_debug_panel(ctx: &egui::Context, state: &mut UiState) {
    egui::SidePanel::right("debug_panel")
        .resizable(true)
        .show(ctx, |ui| {
            ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    debug_panel::create_debug_ui(ui, state);
                });
        });
}
