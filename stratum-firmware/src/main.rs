use std::mem::MaybeUninit;

use esp_idf_sys::{esp, spi_device_transmit, spi_transaction_t};
use lvgl_backend::Esp32LvglBackend;
use stratum_ui_common::{
    amnio_bindings::{self, ui_spi_send_cb_t},
    lvgl_backend::LvglBackend,
};

mod lvgl_backend;
mod stratum_lvgl_ui;

#[no_mangle]
unsafe extern "C" fn handle_spi(is_data: bool, data: *const u8, len: usize) {
    let slice = core::slice::from_raw_parts(data, len);
    // println!("SPI {}: {:?}", if is_data { "DATA" } else { "CMD" }, slice);
    // let spi = *SPI_HANDLE.get(); // it's just a raw pointer now
    let spi = lvgl_backend::SPI_DEV.unwrap();
    let mut dc = lvgl_backend::DC_PIN.get().unwrap().lock().unwrap();

    // 3a) set DC pin
    if is_data {
        dc.set_high().unwrap();
    } else {
        dc.set_low().unwrap();
    }

    // 3b) build an esp-idf transaction
    let mut trans: spi_transaction_t = MaybeUninit::zeroed().assume_init();
    trans.length = (len * 8) as usize;
    trans.__bindgen_anon_1.tx_buffer = data as *const _;

    // 3c) send it!
    esp!(spi_device_transmit(spi, &mut trans)).unwrap();
}

fn main() {
    esp_idf_svc::sys::link_patches();

    let backend = Esp32LvglBackend;
    let cb: ui_spi_send_cb_t = Some(handle_spi);
    unsafe {
        amnio_bindings::lvgl_register_spi_send_cb(cb);
    }
    backend.setup_ui();

    loop {
        backend.update_ui();
        // led.set_high().unwrap();
        // FreeRtos::delay_ms(DELAY);
        // led.set_low().unwrap();
        // FreeRtos::delay_ms(DELAY);
    }
}
