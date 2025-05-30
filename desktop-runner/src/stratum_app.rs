use crate::{
    debug_panel,
    hot_reload_manager::SharedHotReloadManager,
    state::{update_fps, CanvasView, UiState},
    stratum_lvgl_ui::StratumLvglUI,
};
use eframe::{egui, CreationContext, Frame};
use egui::{
    epaint::text::{FontInsert, InsertFontFamily},
    Color32, Direction, DragValue, Layout, Pos2, Rect, Response, ScrollArea, Sense, Stroke,
    TextureHandle, Vec2,
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

                ui.label("Zoom:");

                let initial_zoom = (self.ui_state.canvas_view.zoom * 100.0).round() / 100.0;
                let mut zoom_pct = self
                    .ui_state
                    .canvas_view
                    .pending_zoom
                    .unwrap_or(initial_zoom)
                    * 100.0;

                // 2) Draw the drag‐value widget
                let resp = ui.add(
                    DragValue::new(&mut zoom_pct)
                        .range((ZOOM_MIN * 100.0)..=(ZOOM_MAX * 100.0))
                        .speed(1.0)
                        .suffix("%"),
                );

                let new_zoom = (zoom_pct as f32) / 100.0;

                // 3) While the widget has focus, stash whatever they type/drag
                if resp.has_focus() {
                    self.ui_state.canvas_view.pending_zoom = Some(new_zoom);
                }

                // 4) Commit on drag-release OR text-entry focus-loss
                if resp.dragged() || resp.lost_focus() {
                    self.ui_state.canvas_view.zoom = new_zoom.clamp(ZOOM_MIN, ZOOM_MAX);
                    // clear the pending buffer
                    self.ui_state.canvas_view.pending_zoom = None;
                }

                // 5) Your Reset button stays the same
                if ui
                    .add_enabled(
                        (self.ui_state.canvas_view.zoom - 1.0).abs() > f32::EPSILON,
                        egui::Button::new("Reset"),
                    )
                    .clicked()
                {
                    self.ui_state.canvas_view.reset_zoom();
                    self.ui_state.canvas_view.reset_position();
                }
                ui.separator();

                if ui
                    .add_enabled(
                        self.ui_state.canvas_view.offset != Default::default(),
                        egui::Button::new("Re-center"),
                    )
                    .clicked()
                {
                    self.ui_state.canvas_view.reset_position()
                }

                ui.separator();
            });
        });

        // Update FPS counter and loop
        update_fps(&mut self.ui_state, &self.last_frame_start);
        self.last_frame_start = std::time::Instant::now();
        ctx.request_repaint();
    }
}

const ZOOM_MIN: f32 = 0.1;
const ZOOM_MAX: f32 = 200.0;

// ---------- helpers ----------

pub fn draw_lvgl_canvas(ui: &mut egui::Ui, tex: Option<&TextureHandle>, view: &mut CanvasView) {
    // 1) Figure out native LVGL size
    let display_w = unsafe { stratum_ui_common::stratum_ui_ffi::get_lvgl_display_width() as f32 };
    let display_h = unsafe { stratum_ui_common::stratum_ui_ffi::get_lvgl_display_height() as f32 };

    // 2) Grab available rect once
    let avail = ui.available_rect_before_wrap();

    // 3) Handle scroll‐to‐zoom BEFORE computing rect so everything
    //    uses the new zoom/offset in the same frame
    if let Some(cursor) = ui
        .ctx()
        .input(|i| i.pointer.hover_pos())
        .filter(|pos| avail.contains(*pos))
    {
        let scroll = ui.ctx().input(|i| i.raw_scroll_delta.y);
        if scroll != 0.0 {
            // compute new zoom
            let old_zoom = view.zoom;
            let new_zoom = (old_zoom * (1.0 + scroll * 0.001)).clamp(ZOOM_MIN, ZOOM_MAX);
            view.zoom = new_zoom;

            // figure out which LVGL‐pixel was under cursor before zoom
            let world_pixel = (cursor
                - (avail.center() + view.offset
                    - Vec2::new(display_w, display_h) * old_zoom * 0.5))
                / old_zoom;

            // after zoom, we want world_pixel to still sit under cursor
            let scaled = Vec2::new(display_w, display_h) * new_zoom;
            let new_top_left = cursor - world_pixel * new_zoom;
            view.offset = new_top_left + scaled * 0.5 - avail.center();
        }
    }

    // 4) Compute scaled size & snapped top-left
    let scaled = Vec2::new(display_w, display_h) * view.zoom;
    let unrounded_tl = avail.center() + view.offset - scaled * 0.5;
    let top_left = unrounded_tl.round(); // snap to pixel grid
    let rect = Rect::from_min_size(top_left, scaled);

    // 5) Allocate for both hover (zoom) and drag sense
    let response: Response = ui.allocate_rect(rect, Sense::click_and_drag());

    // 6) Panning (drag)
    if response.dragged() {
        view.offset += response.drag_delta();
    }

    // 7) Draw the texture (hover/drag highlight)
    if let (Some(tex), true) = (tex, response.hovered() || response.dragged()) {
        ui.painter().image(
            tex.id(),
            rect,
            Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
            egui::Color32::WHITE,
        );
    } else if let Some(tex) = tex {
        // normal draw
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

    // 8) Draw pixel grid at high zoom
    if view.zoom > 8.0 {
        let painter = ui.painter();
        let stroke = Stroke::new(1.0, Color32::from_rgb(40, 40, 40));

        let cols = (display_w as usize) + 1;
        for col in 0..=cols {
            let x = (rect.left() + (col as f32) * view.zoom).round();
            painter.line_segment(
                [Pos2::new(x, rect.top()), Pos2::new(x, rect.bottom())],
                stroke,
            );
        }

        let rows = (display_h as usize) + 1;
        for row in 0..=rows {
            let y = (rect.top() + (row as f32) * view.zoom).round();
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
