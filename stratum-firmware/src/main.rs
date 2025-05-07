use esp_idf_hal::{delay::FreeRtos, gpio::*, prelude::Peripherals};
use lvgl_backend::Esp32LvglBackend;
use stratum_ui_common::{amnio_bindings, lvgl_backend::LvglBackend};

mod lvgl_backend;
mod stratum_lvgl_ui;

pub struct LvglBuffer {
    buffer: Box<[u16]>,
}

impl LvglBuffer {
    pub fn new() -> Self {
        let byte_size = unsafe { amnio_bindings::lvgl_get_required_framebuffer_size() as usize };
        let word_len = byte_size / core::mem::size_of::<u16>();

        let buffer = vec![0u16; word_len].into_boxed_slice();

        unsafe {
            amnio_bindings::lvgl_register_external_buffer(buffer.as_ptr() as *mut u16, byte_size);
        }

        Self { buffer }
    }

    pub fn as_slice(&self) -> &[u16] {
        &self.buffer
    }

    pub fn as_mut_ptr(&mut self) -> *mut u16 {
        self.buffer.as_mut_ptr()
    }
}

fn main() {
    esp_idf_svc::sys::link_patches();
    let peripherals = Peripherals::take().unwrap();

    let mut led = PinDriver::output(peripherals.pins.gpio2).unwrap();
    const DELAY: u32 = 500;

    let backend = Esp32LvglBackend;
    LvglBuffer::new();
    backend.setup_ui();
    let free = unsafe { esp_idf_sys::heap_caps_get_free_size(esp_idf_sys::MALLOC_CAP_DEFAULT) };
    println!("Free heap: {}", free);

    loop {
        backend.update_ui();
        led.set_high().unwrap();
        FreeRtos::delay_ms(DELAY);
        led.set_low().unwrap();
        FreeRtos::delay_ms(DELAY);
    }
}
