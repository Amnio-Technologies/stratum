use std::{
    thread,
    time::{Duration, Instant},
};

use stratum_ui_common::{amnio_bindings, lvgl_backend::LvglBackend};

pub struct DesktopLvglBackend;

const WIDTH: usize = 320;
const HEIGHT: usize = 240;

// 💾 Static framebuffer lives for the entire program
static mut FRAMEBUFFER: [u16; WIDTH * HEIGHT] = [0; WIDTH * HEIGHT];

impl LvglBackend for DesktopLvglBackend {
    fn setup_ui(&self) {
        unsafe {
            amnio_bindings::lvgl_register_external_buffer(
                FRAMEBUFFER.as_mut_ptr(),
                WIDTH * HEIGHT * 2,
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
                    amnio_bindings::lvgl_advance_timer(elapsed_ms);
                }

                thread::sleep(Duration::from_millis(5));
            }
        });
    }

    fn update_ui(&self) {
        unsafe {
            amnio_bindings::lvgl_update();
        }
    }
}
