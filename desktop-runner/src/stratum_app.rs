use crate::{
    debug_ui,
    hot_reload_manager::SharedHotReloadManager,
    state::{update_fps, UiState},
    stratum_lvgl_ui::StratumLvglUI,
};
use eframe::{egui, CreationContext, Frame};
use egui::{Direction, Layout, ScrollArea, TextureHandle};
use std::sync::{atomic::Ordering, Arc};
use stratum_ui_common::ui_logging::UiLogger;

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
    ) -> Self {
        let ui_state = UiState::new(cc, ui_logger, hot_reload_manager);
        let lvgl_ui = StratumLvglUI::new();

        Self {
            ui_state,
            lvgl_ui,
            lvgl_tex: None,
            last_frame_start: std::time::Instant::now(),
        }
    }
}

impl eframe::App for StratumApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut Frame) {
        // Hot-reload check
        if self
            .ui_state
            .hot_reload_manager
            .lock()
            .unwrap()
            .should_reload_ui
            .swap(false, Ordering::Relaxed)
        {
            self.ui_state.ui_logger.clone().bind_callback();
            self.lvgl_ui.reload_ui();
        }

        // Generate the latest LVGL texture
        self.lvgl_tex = self.lvgl_ui.update(ctx).cloned();

        draw_debug_panel(ctx, &mut self.ui_state);

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.with_layout(
                Layout::centered_and_justified(Direction::LeftToRight),
                |ui| {
                    draw_lvgl_canvas(ui, self.lvgl_tex.as_ref());
                },
            );
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
                    debug_ui::create_debug_ui(ui, state);
                });
        });
}
