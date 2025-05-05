use crate::amnio_bindings;

pub trait LvglBackend {
    fn setup_ui(&self);
    fn update_ui(&self);

    fn get_framebuffer(&self) -> Option<(&'static mut [u16], usize, usize)> {
        let ptr = unsafe { amnio_bindings::get_lvgl_framebuffer() };
        if ptr.is_null() {
            return None;
        }

        let width = unsafe { amnio_bindings::get_lvgl_display_width() } as usize;
        let height = unsafe { amnio_bindings::get_lvgl_display_height() } as usize;

        Some(unsafe {
            (
                std::slice::from_raw_parts_mut(ptr, width * height),
                width,
                height,
            )
        })
    }
}
