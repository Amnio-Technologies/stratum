use crate::{
    debug_panel,
    hot_reload_manager::SharedHotReloadManager,
    state::{update_fps, CanvasView, UiState},
    stratum_lvgl_ui::StratumLvglUI,
};
use eframe::{egui, CreationContext, Frame};
use egui::{
    epaint::text::{FontInsert, InsertFontFamily},
    Color32, Direction, Layout, Pos2, Rect, Response, ScrollArea, Sense, Stroke, TextureHandle,
    Vec2,
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
                        draw_lvgl_canvas(
                            ui,
                            self.lvgl_tex.as_ref(),
                            &mut self.ui_state.canvas_view,
                        );
                    },
                );
            });

        egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(format!("FPS: {:.2}", self.ui_state.fps));
                ui.separator();
                ui.label(format!(
                    "Zoom: {:.0}%",
                    self.ui_state.canvas_view.zoom * 100.0
                ));
            });
        });

        // Update FPS counter and loop
        update_fps(&mut self.ui_state, &self.last_frame_start);
        self.last_frame_start = std::time::Instant::now();
        ctx.request_repaint();
    }
}

// ---------- helpers ----------

pub fn draw_lvgl_canvas(ui: &mut egui::Ui, tex: Option<&TextureHandle>, view: &mut CanvasView) {
    // 1) Figure out native LVGL size
    let display_w = unsafe { stratum_ui_common::stratum_ui_ffi::get_lvgl_display_width() as f32 };
    let display_h = unsafe { stratum_ui_common::stratum_ui_ffi::get_lvgl_display_height() as f32 };

    // 2) Compute scaled size
    let scaled = Vec2::new(display_w, display_h) * view.zoom;

    // 3) Center + pan
    let avail = ui.available_rect_before_wrap();
    let top_left = avail.center() + view.offset - scaled * 0.5;
    let rect = Rect::from_min_size(top_left, scaled);

    // 4) Allocate with both hover (for zoom) and drag sense
    let response: Response = ui.allocate_rect(rect, Sense::click_and_drag());

    // 5) Zooming (scroll) — only when hovering
    if response.hovered() {
        let scroll = ui.ctx().input(|i| i.raw_scroll_delta.y);
        if scroll != 0.0 {
            // adjust zoom (clamped)
            let factor = (1.0 + scroll * 0.001).clamp(0.1, 10.0);
            view.zoom *= factor;

            // optional: zoom about cursor
            let cursor = ui
                .ctx()
                .input(|i| i.pointer.hover_pos())
                .unwrap_or(avail.center());
            let to_cursor = cursor - rect.center();
            view.offset = (view.offset + to_cursor) * factor - to_cursor;
        }
    }

    // 6) Panning (drag)
    if response.dragged() {
        view.offset += response.drag_delta();
    }

    // 7) Finally draw the texture
    if let (Some(tex), true) = (tex, response.hovered() || response.dragged()) {
        ui.painter().image(
            tex.id(),
            rect,
            Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
            egui::Color32::WHITE,
        );
    } else if let Some(tex) = tex {
        // draw even when idle
        ui.painter().image(
            tex.id(),
            rect,
            Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
            egui::Color32::WHITE,
        );
    } else {
        ui.painter().text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            "No LVGL texture",
            egui::FontId::proportional(16.0),
            egui::Color32::GRAY,
        );
    }

    if view.zoom > 8.0 {
        let painter = ui.painter();

        // line style: thin & semi-transparent white
        let stroke = Stroke::new(1.0, Color32::from_rgb(40, 40, 40));

        // draw vertical pixel lines every 1 original px
        let cols = (display_w as usize) + 1;
        for col in 0..=cols {
            let x = rect.left() + (col as f32) * view.zoom;
            painter.line_segment(
                [Pos2::new(x, rect.top()), Pos2::new(x, rect.bottom())],
                stroke,
            );
        }

        // draw horizontal pixel lines
        let rows = (display_h as usize) + 1;
        for row in 0..=rows {
            let y = rect.top() + (row as f32) * view.zoom;
            painter.line_segment(
                [Pos2::new(rect.left(), y), Pos2::new(rect.right(), y)],
                stroke,
            );
        }
    }
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
