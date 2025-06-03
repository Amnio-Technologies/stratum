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
    /// Call this once before your render loop starts, to initialize.
    pub fn new() -> Self {
        Self {
            fps: 0.0,
            frame_count: 0,
            last_update: Instant::now(),
        }
    }

    /// Call this at the very end of each frame, passing in the
    /// Instant you captured at the start of that same frame.
    ///
    /// Returns the updated FPS. Also, if the frame ran faster
    /// than ~16ms, it will sleep for the remainder to cap at ~60 FPS.
    pub fn tick(&mut self) -> f64 {
        // 1) Count this frame
        self.frame_count += 1;
        let since = self.last_update.elapsed();

        // 2) Once per second, update `fps` and reset the counter
        if since >= Duration::from_secs(1) {
            self.fps = self.frame_count as f64 / since.as_secs_f64();
            self.frame_count = 0;
            self.last_update = Instant::now();
        }

        // No more sleep here—frame pacing is handled by request_repaint_after().
        self.fps
    }
}
