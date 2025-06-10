use egui::{Pos2, Rect};
use std::{
    os::raw::c_void,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
    time::{Duration, Instant},
};
use stratum_ui_common::stratum_ui_ffi::{self, lv_area_t};

/// Convert the C LVGL area into an egui Rect
fn lv_area_to_egui_rect(a: lv_area_t) -> Rect {
    Rect {
        min: Pos2::new(a.x1 as f32, a.y1 as f32),
        max: Pos2::new(a.x2 as f32, a.y2 as f32),
    }
}

/// One merged‐rect event tagged with its timestamp.
#[derive(Debug, Clone)]
pub struct FrameRect {
    pub timestamp: Instant,
    pub rects: Vec<Rect>,
}

/// Thread‐safe collector for LVGL flush areas, with a `.scope()` API.
pub struct FlushAreaCollector {
    /// Raw per‐scope rectangles from the C callback
    areas: Mutex<Vec<Rect>>,
    /// Merged+timestamped events you can poll()
    events: Mutex<Vec<FrameRect>>,
    /// Whether flashing is enabled
    enabled: AtomicBool,
}

pub const FLASH_DURATION: Duration = Duration::from_millis(1000 / 3);

impl FlushAreaCollector {
    /// Create the collector, register the FFI callback, and return an Arc.
    pub fn new(enabled: bool) -> Arc<Self> {
        let coll = Arc::new(FlushAreaCollector {
            areas: Mutex::new(Vec::new()),
            events: Mutex::new(Vec::new()),
            enabled: AtomicBool::new(enabled),
        });
        // bind our C callback
        coll.clone().bind();
        coll
    }

    /// Clear raw LVGL‐reported rects before each scope.
    fn clear(&self) {
        self.areas.lock().unwrap().clear();
    }

    /// Consume & return the raw rects collected since the last clear.
    fn take_raw(&self) -> Vec<Rect> {
        std::mem::take(&mut *self.areas.lock().unwrap())
    }

    /// Register/re-register the C flush callback.
    pub fn bind(self: Arc<Self>) {
        unsafe {
            stratum_ui_ffi::register_flush_area_cb(
                Some(flush_area_cb),
                Arc::as_ptr(&self) as *mut c_void,
            );
        }
    }

    /// Run one “frame” of LVGL work, merge that frame’s rects,
    /// timestamp them, and store in our internal buffer.
    pub fn scope<F: FnOnce()>(&self, f: F) {
        self.clear();

        f();

        if !self.is_enabled() {
            return;
        }

        let raw = self.take_raw();

        if raw.is_empty() {
            return;
        }

        let merged = Self::merge_rects(raw);

        self.cull_outdated_events();

        let mut ev = self.events.lock().unwrap();
        ev.push(FrameRect {
            timestamp: Instant::now(),
            rects: merged,
        });
    }

    /// Check if flashing is enabled.
    pub fn is_enabled(&self) -> bool {
        self.enabled.load(Ordering::Relaxed)
    }

    /// Enable or disable flashing.
    pub fn set_enabled(&self, enabled: bool) {
        self.enabled.store(enabled, Ordering::Relaxed);
    }

    fn cull_outdated_events(&self) {
        let mut ev = self.events.lock().unwrap();

        ev.retain(|e| e.timestamp.elapsed() < FLASH_DURATION);
    }

    pub fn active_events(&self) -> Vec<FrameRect> {
        if !self.is_enabled() {
            return Vec::new();
        }

        self.cull_outdated_events();
        self.events.lock().unwrap().clone()
    }

    /// Naive merge: returns one bounding‐box that covers them all.
    fn merge_rects(raw: Vec<Rect>) -> Vec<Rect> {
        if raw.is_empty() {
            Vec::new()
        } else {
            let mut x1 = raw[0].min.x;
            let mut y1 = raw[0].min.y;
            let mut x2 = raw[0].max.x;
            let mut y2 = raw[0].max.y;
            for r in &raw {
                x1 = x1.min(r.min.x);
                y1 = y1.min(r.min.y);
                x2 = x2.max(r.max.x);
                y2 = y2.max(r.max.y);
            }
            vec![Rect {
                min: Pos2::new(x1, y1),
                max: Pos2::new(x2, y2),
            }]
        }
    }
}

unsafe extern "C" fn flush_area_cb(user_data: *mut c_void, area: *const lv_area_t) {
    if user_data.is_null() || area.is_null() {
        return;
    }
    // Recover our Arc<FlushAreaCollector>
    let ptr = user_data as *const FlushAreaCollector;
    Arc::increment_strong_count(ptr);
    let collector: Arc<FlushAreaCollector> = Arc::from_raw(ptr);

    // Convert via helper & push
    let lv_area = unsafe { *area };
    let rect = lv_area_to_egui_rect(lv_area);
    collector.areas.lock().unwrap().push(rect);

    // Balance the strong count
    Arc::into_raw(collector);
}
