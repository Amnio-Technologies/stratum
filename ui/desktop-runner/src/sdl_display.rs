use lvgl::draw::{DisplayDriver, DrawBuffer};
use lvgl::{Color, Display, Lvgl};
use sdl2::{pixels::PixelFormatEnum, render::Canvas, video::Window};
use std::thread::sleep;
use std::time::Duration;

/// SDL2 Display Backend for LVGL UI in `desktop-runner`
pub struct SDLDisplay {
    lvgl: Lvgl,
    canvas: Canvas<Window>,
    buffer: DrawBuffer<{ 240 * 240 }>, // LVGL framebuffer
}

impl SDLDisplay {
    /// Initializes the SDL-based UI system.
    pub fn new() -> Self {
        let mut lvgl = Lvgl::init();

        let sdl_context = sdl2::init().unwrap();
        let video_subsystem = sdl_context.video().unwrap();
        let window = video_subsystem
            .window("amnIO UI Emulator", 240, 240)
            .position_centered()
            .opengl()
            .build()
            .unwrap();

        let canvas = window.into_canvas().software().build().unwrap();

        // Create an LVGL display buffer
        let buffer = DrawBuffer::<{ 240 * 240 }>::default();
        let disp_drv = DisplayDriver::new(buffer);
        let _display = lvgl.register_display(disp_drv);

        Self {
            lvgl,
            canvas,
            buffer,
        }
    }

    /// Loads a test blue screen.
    pub fn load_blue_screen(&mut self) {
        let screen = self.lvgl.default_display().screen();
        screen.set_style_bg_color(Color::from_rgb((0, 0, 255))); // Blue background
    }

    /// Provides access to the raw framebuffer pixels for rendering.
    pub fn get_framebuffer(&self) -> &[u8] {
        self.buffer.as_ref()
    }

    /// Updates the UI.
    pub fn update(&mut self) {
        self.lvgl.task_handler(); // Update LVGL
        self.canvas.present(); // Render SDL2 window
        sleep(Duration::from_millis(16)); // ~60FPS
    }
}
