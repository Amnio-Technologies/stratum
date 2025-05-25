use std::{
    cell::UnsafeCell,
    thread,
    time::{Duration, Instant},
};

use stratum_ui_common::{amnio_bindings, lvgl_backend::LvglBackend};

const WIDTH: usize = 320;
const HEIGHT: usize = 240;

pub struct DesktopLvglBackend {
    // May be mutated by LVGL at any time, must live in an UnsafeCell
    frame_buffer: UnsafeCell<[u16; WIDTH * HEIGHT]>,
}

// Safety: it’s OK to share this across threads because LVGL is the only writer
unsafe impl Sync for DesktopLvglBackend {}

impl DesktopLvglBackend {
    pub fn new() -> Self {
        Self {
            frame_buffer: UnsafeCell::new([0; WIDTH * HEIGHT]),
        }
    }

    /// Stable, mutable pointer LVGL can write into.
    pub fn fb_ptr(&self) -> *mut u16 {
        self.frame_buffer.get() as *mut u16
    }

    /// Provides a read-only snapshot of the current framebuffer
    pub fn with_framebuffer<R>(&self, f: impl FnOnce(&[u16]) -> R) -> R {
        // Safety: we create a fresh slice; it lives only inside `f`
        let slice = unsafe {
            core::slice::from_raw_parts(self.frame_buffer.get() as *const u16, WIDTH * HEIGHT)
        };
        f(slice)
    }
}

impl LvglBackend for DesktopLvglBackend {
    fn setup_ui(&mut self) {
        unsafe {
            amnio_bindings::lvgl_register_external_buffer(
                self.fb_ptr(),
                WIDTH * HEIGHT * size_of::<u16>(),
            );
            amnio_bindings::lvgl_setup();
        }

        thread::spawn(move || {
            let mut last_update = Instant::now();

            loop {
                let now = Instant::now();
                let elapsed_ms = now.duration_since(last_update).as_millis() as u32;
                last_update = now;

                unsafe {
                    amnio_bindings::lvgl_update(elapsed_ms);
                }

                thread::sleep(Duration::from_millis(5));
            }
        });
    }

    fn update_ui(&mut self) {
        unsafe {
            // amnio_bindings::lvgl_update();
        }
    }
}
