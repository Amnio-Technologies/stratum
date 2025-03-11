#![allow(non_snake_case)]

use std::ffi::c_void;

#[link(name = "amnio_ui")] // Link against `libamnio_ui.a`
extern "C" {
    pub fn lvgl_setup();
    pub fn lvgl_update();
    pub fn get_lvgl_framebuffer() -> *mut u16;
}

// 🟢 Rust-friendly functions
pub fn setup_ui() {
    unsafe { lvgl_setup() };
}

pub fn update_ui() {
    unsafe { lvgl_update() };
}

pub fn get_framebuffer() -> &'static mut [u16] {
    let ptr = unsafe { get_lvgl_framebuffer() };
    unsafe { std::slice::from_raw_parts_mut(ptr, 480 * 320) }
}

// 🟢 Example: Calling C Functions from Rust
fn main() {
    setup_ui();
    loop {
        update_ui();
        let framebuffer = get_framebuffer();
        println!("Framebuffer First Pixel: {}", framebuffer[0]);
    }
}
