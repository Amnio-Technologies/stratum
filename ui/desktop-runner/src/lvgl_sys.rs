use std::ptr;

#[link(name = "lvgl_c", kind = "dylib")]
extern "C" {
    fn lvgl_setup();
    fn lvgl_update();
    fn get_lvgl_framebuffer() -> *const u16;
}

/// Initializes LVGL (calls C function)
pub fn init_lvgl() {
    unsafe { lvgl_setup() };
}

/// Steps LVGL (calls C function)
pub fn update_lvgl() {
    unsafe { lvgl_update() };
}

/// Gets a reference to the LVGL framebuffer
pub fn get_framebuffer() -> &'static [u16] {
    unsafe {
        let ptr = get_lvgl_framebuffer();
        if ptr.is_null() {
            return &[];
        }
        std::slice::from_raw_parts(ptr, 480 * 320)
    }
}
