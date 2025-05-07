use esp_idf_hal::{delay::FreeRtos, gpio::*, prelude::Peripherals};
use lvgl_backend::Esp32LvglBackend;
use stratum_ui_common::lvgl_backend::LvglBackend;

mod lvgl_backend;
mod stratum_lvgl_ui;

fn main() {
    esp_idf_svc::sys::link_patches();
    let peripherals = Peripherals::take().unwrap();

    let mut led = PinDriver::output(peripherals.pins.gpio2).unwrap();
    const DELAY: u32 = 500;

    let backend = Esp32LvglBackend;
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
