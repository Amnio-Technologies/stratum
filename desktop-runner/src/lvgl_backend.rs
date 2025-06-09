use std::mem::size_of;
use std::{cell::UnsafeCell, time::Instant};

use stratum_ui_common::{lvgl_backend::LvglBackend, stratum_ui_ffi};

const WIDTH: usize = 320;
const HEIGHT: usize = 240;

pub struct DesktopLvglBackend {
    frame_buffer: UnsafeCell<[u16; WIDTH * HEIGHT]>,
    last_update: Instant,
}

// Safety: LVGL is the only writer; we only expose shared references safely.
unsafe impl Sync for DesktopLvglBackend {}

impl DesktopLvglBackend {
    pub fn new() -> Self {
        Self {
            frame_buffer: UnsafeCell::new([0; WIDTH * HEIGHT]),
            last_update: Instant::now(),
        }
    }

    /// Stable, mutable pointer LVGL can write into.
    pub fn fb_ptr(&self) -> *mut u16 {
        self.frame_buffer.get() as *mut u16
    }

    /// Read-only framebuffer snapshot.
    pub fn with_framebuffer<R>(&self, f: impl FnOnce(&[u16]) -> R) -> R {
        let slice = unsafe {
            core::slice::from_raw_parts(self.frame_buffer.get() as *const u16, WIDTH * HEIGHT)
        };
        f(slice)
    }
}

impl LvglBackend for DesktopLvglBackend {
    fn setup_ui(&mut self) {
        unsafe {
            stratum_ui_ffi::lvgl_register_external_buffer(
                self.fb_ptr(),
                WIDTH * HEIGHT * size_of::<u16>(),
            );
            stratum_ui_ffi::lvgl_setup();
        }

        self.last_update = Instant::now(); // Reset timer
    }

    fn update_ui(&mut self) {
        let now = Instant::now();
        let dt = now.duration_since(self.last_update);
        self.last_update = now;

        let elapsed_ms = dt.as_millis() as u32; // Cap to ~30fps for stability
        unsafe {
            stratum_ui_ffi::lvgl_update(elapsed_ms);
        }
    }
}
