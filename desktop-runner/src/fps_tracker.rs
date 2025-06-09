use std::time::{Duration, Instant};

/// Keeps track of frames per second and optionally sleeps
/// to cap at ~60 Hz.
pub struct FpsTracker {
    fps: f64,
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

    /// Call once per frame. Returns true if the FPS value was recalculated.
    pub fn tick(&mut self) -> bool {
        self.frame_count += 1;
        let now = Instant::now();
        let elapsed = now - self.last_update;

        if elapsed >= Duration::from_secs(1) {
            self.fps = self.frame_count as f64 / elapsed.as_secs_f64();
            self.frame_count = 0;
            // Advance by *exactly* the elapsed time to avoid drift:
            self.last_update += elapsed;
            true
        } else {
            false
        }
    }

    /// Read the last computed FPS.
    pub fn fps(&self) -> f64 {
        self.fps
    }
}
