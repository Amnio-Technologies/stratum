use std::sync::Once;
use std::time::Duration;
use std::time::Instant;

use esp_idf_hal::gpio::Output;
use esp_idf_hal::gpio::Pin;
use esp_idf_hal::gpio::PinDriver;
use esp_idf_hal::prelude::Peripherals;
use esp_idf_hal::sys::spi_bus_config_t;
use esp_idf_hal::sys::spi_bus_initialize;
use esp_idf_hal::sys::spi_device_handle_t;
use esp_idf_hal::sys::spi_device_transmit;
use esp_idf_hal::sys::spi_transaction_t;
use esp_idf_sys::esp;
use esp_idf_sys::spi_bus_add_device;
use esp_idf_sys::spi_device_interface_config_t;
use esp_idf_sys::spi_host_device_t_SPI2_HOST;
use stratum_ui_common::amnio_bindings;
use stratum_ui_common::amnio_bindings::{get_lvgl_display_height, get_lvgl_display_width};
use stratum_ui_common::lvgl_backend::LvglBackend;

pub struct FirmwareLvglBackend;

impl LvglBackend for FirmwareLvglBackend {
    fn setup_ui(&self) {
        // In embedded context, likely sets up LVGL + display + input drivers
        unsafe {
            amnio_bindings::lvgl_setup(); // Or a native setup()
        }
    }

    fn update_ui(&self) {
        unsafe {
            amnio_bindings::lvgl_update(); // Typically calls lv_task_handler()
        }
    }
}

// fn something() {
//     let peripherals = Peripherals::take().unwrap();
//     let pins = peripherals.pins;
//     let sclk = pins.gpio18;
//     let mosi = pins.gpio23;
//     let cs = pins.gpio5;
//     let dc = pins.gpio2;
//     let rst = pins.gpio4;
// }

fn get_max_transfer_size() -> i32 {
    unsafe {
        (get_lvgl_display_width() * get_lvgl_display_height() * 2)
            .try_into()
            .unwrap()
    }
}

// You'll store this globally after `spi_bus_add_device()`
static mut SPI_DEV: Option<spi_device_handle_t> = None;

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

    esp!(spi_bus_initialize(spi_host_device_t_SPI2_HOST, &bus_cfg, 1)).unwrap();

    let dev_cfg = spi_device_interface_config_t {
        command_bits: 0,
        address_bits: 0,
        dummy_bits: 0,
        mode: 0,
        duty_cycle_pos: 128,
        cs_ena_posttrans: 0,
        cs_ena_pretrans: 0,
        clock_speed_hz: 40_000_000,
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
        spi_host_device_t_SPI2_HOST,
        &dev_cfg,
        &mut handle
    ))
    .unwrap();

    SPI_DEV = Some(handle);
}

pub unsafe fn flush_st7789<'d, T: Pin>(framebuffer: &[u16], dc: &mut PinDriver<'d, T, Output>) {
    let spi = SPI_DEV.expect("SPI device not initialized");

    // --- Step 1: Send RAMWR command (0x2C) ---
    dc.set_low().unwrap(); // Command mode

    let cmd = [0x2C];
    let mut trans_cmd: spi_transaction_t = core::mem::zeroed();
    trans_cmd.length = 8; // bits
    trans_cmd.__bindgen_anon_1.tx_buffer = cmd.as_ptr() as *const _;

    esp!(spi_device_transmit(spi, &mut trans_cmd)).unwrap();

    // --- Step 2: Send pixel data ---
    dc.set_high().unwrap(); // Data mode

    let byte_len = framebuffer.len() * 2;
    let mut trans_data: spi_transaction_t = core::mem::zeroed();
    trans_data.length = (byte_len * 8) as usize; // bits
    trans_data.__bindgen_anon_1.tx_buffer = framebuffer.as_ptr() as *const _;

    esp!(spi_device_transmit(spi, &mut trans_data)).unwrap();
}

pub struct Esp32LvglBackend;

static INIT: Once = Once::new();

impl LvglBackend for Esp32LvglBackend {
    fn setup_ui(&self) {
        INIT.call_once(|| unsafe {
            init_spi_bus();
            amnio_bindings::lvgl_setup();
            spawn_lvgl_timer_task();
        });
    }

    fn update_ui(&self) {
        unsafe {
            amnio_bindings::lvgl_update();

            // Convert framebuffer to slice
            let fb_ptr = amnio_bindings::get_lvgl_framebuffer();
            let fb_len = (amnio_bindings::get_lvgl_display_width()
                * amnio_bindings::get_lvgl_display_height()) as usize;
            let framebuffer: &[u16] = core::slice::from_raw_parts(fb_ptr, fb_len);

            // Re-acquire peripherals and configure DC pin
            let peripherals = Peripherals::take().unwrap();
            let dc_pin = PinDriver::output(peripherals.pins.gpio2).unwrap();

            // Note: dc_pin needs to be mutable, so make it `mut`
            let mut dc_pin = dc_pin;

            flush_st7789(framebuffer, &mut dc_pin);
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
