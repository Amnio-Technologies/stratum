use std::time::{Duration, Instant};

/// Keeps track of frames per second and optionally sleeps
/// to cap at ~60 Hz.
pub struct FpsTracker {
    /// Measured frames per second over the last second.
    pub fps: f64,

    frame_count: u32,
    last_update: Instant,
}

impl FpsTracker {
    pub fn new() -> Self {
        Self {
            fps: 0.0,
            frame_count: 0,
            last_update: Instant::now(),
        }
    }

    pub fn tick(&mut self) -> f64 {
        self.frame_count += 1;
        let since = self.last_update.elapsed();

        if since >= Duration::from_secs(1) {
            self.fps = self.frame_count as f64 / since.as_secs_f64();
            self.frame_count = 0;
            self.last_update = Instant::now();
        }

        self.fps
    }
}
