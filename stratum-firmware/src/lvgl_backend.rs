use std::sync::Mutex;
use std::sync::Once;
use std::sync::OnceLock;
use std::time::Duration;
use std::time::Instant;

use esp_idf_hal::delay::FreeRtos;
use esp_idf_hal::gpio::Gpio2;
use esp_idf_hal::gpio::Output;
use esp_idf_hal::gpio::PinDriver;
use esp_idf_hal::prelude::Peripherals;
use esp_idf_hal::sys::spi_bus_config_t;
use esp_idf_hal::sys::spi_bus_initialize;
use esp_idf_hal::sys::spi_device_handle_t;
use esp_idf_sys::esp;
use esp_idf_sys::spi_bus_add_device;
use esp_idf_sys::spi_device_interface_config_t;
use esp_idf_sys::spi_host_device_t_SPI3_HOST;
use stratum_ui_common::amnio_bindings;
use stratum_ui_common::amnio_bindings::{get_lvgl_display_height, get_lvgl_display_width};
use stratum_ui_common::lvgl_backend::LvglBackend;

use crate::handle_spi;

pub struct FirmwareLvglBackend;

impl LvglBackend for FirmwareLvglBackend {
    fn setup_ui(&self) {
        unsafe {
            amnio_bindings::lvgl_setup();
        }
    }

    fn update_ui(&self) {
        unsafe {
            amnio_bindings::lvgl_update();
        }
    }
}

fn get_max_transfer_size() -> i32 {
    unsafe {
        (get_lvgl_display_width() * get_lvgl_display_height() * 2)
            .try_into()
            .unwrap()
    }
}

// You'll store this globally after `spi_bus_add_device()`
pub static mut SPI_DEV: Option<spi_device_handle_t> = None;
// static mut SPI_HANDLE: UnsafeCell<spi_device_handle_t> = UnsafeCell::new(core::ptr::null_mut());
pub static DC_PIN: OnceLock<Mutex<PinDriver<'static, Gpio2, Output>>> = OnceLock::new();

unsafe fn init_spi_bus() {
    let mut bus_cfg: spi_bus_config_t = core::mem::zeroed();

    bus_cfg.__bindgen_anon_1.mosi_io_num = 23;
    bus_cfg.__bindgen_anon_2.miso_io_num = -1; // not used
    bus_cfg.sclk_io_num = 18;
    bus_cfg.__bindgen_anon_3.quadwp_io_num = -1;
    bus_cfg.__bindgen_anon_4.quadhd_io_num = -1;

    bus_cfg.max_transfer_sz = get_max_transfer_size(); // full-screen RGB565
    bus_cfg.intr_flags = 0; // default
    bus_cfg.flags = 0; // no special flags

    esp!(spi_bus_initialize(spi_host_device_t_SPI3_HOST, &bus_cfg, 1)).unwrap();

    let dev_cfg = spi_device_interface_config_t {
        command_bits: 0,
        address_bits: 0,
        dummy_bits: 0,
        mode: 0,
        duty_cycle_pos: 128,
        cs_ena_posttrans: 0,
        cs_ena_pretrans: 0,
        clock_speed_hz: 10_000_000,
        input_delay_ns: 0,
        spics_io_num: 5,
        flags: 0,
        queue_size: 1,
        pre_cb: None,
        post_cb: None,
        ..Default::default()
    };

    let mut handle: spi_device_handle_t = core::ptr::null_mut();
    esp!(spi_bus_add_device(
        spi_host_device_t_SPI3_HOST,
        &dev_cfg,
        &mut handle
    ))
    .unwrap();

    SPI_DEV = Some(handle);
}

pub struct Esp32LvglBackend;

static INIT: Once = Once::new();

pub unsafe fn write_cmd(cmd: u8) {
    handle_spi(false, &cmd as *const u8, 1);
}

pub unsafe fn write_data(data: &[u8]) {
    handle_spi(true, data.as_ptr(), data.len());
}

pub unsafe fn init_st7789vw() {
    write_cmd(0x01); // Software reset
    FreeRtos::delay_ms(150);

    write_cmd(0x11); // Sleep out
    FreeRtos::delay_ms(500);

    write_cmd(0x36); // MADCTL: Memory Access Control
    write_data(&[0x00]); // Try 0x00, 0x70, 0xC0 for orientation tweaks

    write_cmd(0x3A); // COLMOD: Pixel Format Set
    write_data(&[0x05]); // 16-bit/pixel

    write_cmd(0xB2); // Porch Setting
    write_data(&[0x0C, 0x0C, 0x00, 0x33, 0x33]);

    write_cmd(0xB7); // Gate Control
    write_data(&[0x35]);

    write_cmd(0xBB); // VCOM Setting
    write_data(&[0x19]);

    write_cmd(0xC0); // Power Control 1
    write_data(&[0x2C]);

    write_cmd(0xC2); // VDV and VRH Control
    write_data(&[0x01]);

    write_cmd(0xC3); // VRH Set
    write_data(&[0x12]);

    write_cmd(0xC4); // VDV Set
    write_data(&[0x20]);

    write_cmd(0xC6); // Frame Rate Control in Normal Mode
    write_data(&[0x0F]);

    write_cmd(0xD0); // Power Control 2
    write_data(&[0xA4, 0xA1]);

    write_cmd(0xE0); // Positive Gamma Correction
    write_data(&[
        0xD0, 0x08, 0x11, 0x08, 0x0C, 0x15, 0x39, 0x33, 0x50, 0x36, 0x13, 0x14, 0x29, 0x2D,
    ]);

    write_cmd(0xE1); // Negative Gamma Correction
    write_data(&[
        0xD0, 0x08, 0x10, 0x08, 0x06, 0x06, 0x39, 0x44, 0x51, 0x0B, 0x16, 0x14, 0x2F, 0x31,
    ]);

    write_cmd(0x21); // Display inversion ON (optional but improves colors)
    write_cmd(0x2A);
    write_data(&[0x00, 0x00, 0x00, 0xEF]); // CASET: 0–239
    write_cmd(0x2B);
    write_data(&[0x00, 0x00, 0x01, 0x3F]); // RASET: 0–319
    write_cmd(0x29); // Display ON
    FreeRtos::delay_ms(100);
}

impl LvglBackend for Esp32LvglBackend {
    fn setup_ui(&self) {
        INIT.call_once(|| unsafe {
            let peripherals = Peripherals::take().unwrap();
            let mut rst = PinDriver::output(peripherals.pins.gpio4).unwrap();
            rst.set_low().unwrap(); // assert reset
            FreeRtos::delay_ms(20);
            rst.set_high().unwrap(); // release reset
            FreeRtos::delay_ms(120); // allow display to stabilize

            let dc_pin = PinDriver::output(peripherals.pins.gpio2).unwrap();

            // 💥 Assign to the OnceLock properly
            DC_PIN.set(Mutex::new(dc_pin));

            init_spi_bus();
            init_st7789vw();

            write_cmd(0x2C);
            let red_pixel = [0xF8, 0x00];
            for _ in 0..(240 * 240) {
                write_data(&red_pixel);
            }

            amnio_bindings::lvgl_setup();
        });
    }

    fn update_ui(&self) {
        unsafe {
            amnio_bindings::lvgl_update();
        }
    }
}

unsafe fn spawn_lvgl_timer_task() {
    // Create a task to call `lvgl_advance_timer()` every 5ms
    std::thread::spawn(move || {
        let mut last_tick = Instant::now();

        loop {
            let now = Instant::now();
            let dt = now.duration_since(last_tick).as_millis() as u32;
            last_tick = now;

            amnio_bindings::lvgl_advance_timer(dt);
            std::thread::sleep(Duration::from_millis(5));
        }
    });
}
