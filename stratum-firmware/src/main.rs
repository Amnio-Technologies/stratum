use esp_idf_hal::delay::FreeRtos;
use esp_idf_hal::gpio::*;
use esp_idf_hal::peripherals::Peripherals;

fn main() {
    esp_idf_svc::sys::link_patches();

    let peripherals = Peripherals::take().unwrap();
    let mut led = PinDriver::output(peripherals.pins.gpio2).unwrap();

    loop {
        led.set_high().unwrap();
        FreeRtos::delay_ms(2_000);
        led.set_low().unwrap();
        FreeRtos::delay_ms(2_000);
    }
}
