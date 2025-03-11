use lvgl::Color;

pub struct EmbeddedDisplay;

impl EmbeddedDisplay {
    pub fn new() -> Self {
        Self
    }
}

impl DisplayBackend for EmbeddedDisplay {
    fn flush(&mut self, buffer: &[Color]) {
        for pixel in buffer {
            let rgb565 = pixel.to_rgb565(); // Convert color to 16-bit format
            self.send_pixel_to_display(rgb565);
        }
    }

    fn present(&mut self) {
        // No-op for hardware, rendering is done as we flush.
    }
}

impl EmbeddedDisplay {
    fn send_pixel_to_display(&self, pixel_data: u16) {
        // SPI or I2C write logic here
    }
}
