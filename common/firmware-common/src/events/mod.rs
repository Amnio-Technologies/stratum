use std::sync::Arc;

use crate::modules::system_controller::ModuleEvent;

pub trait EventQueue: Send + Sync {
    fn send(&self, event: ModuleEvent);
    fn receive(&self) -> Option<ModuleEvent>;
}

/// Automatically creates the right queue based on platform
pub fn create_event_queue() -> impl EventQueue + Send + Sync {
    #[cfg(not(target_arch = "xtensa"))] // Windows/Linux
    return MpscEventQueue::new();

    #[cfg(target_arch = "xtensa")] // ESP32
    return FreeRtosEventQueue::new();
}

#[cfg(not(target_arch = "xtensa"))] // Include this module on Windows/Linux
mod desktop_event_queue;
#[cfg(not(target_arch = "xtensa"))]
pub use desktop_event_queue::MpscEventQueue;

#[cfg(target_arch = "xtensa")] // Include this module only on ESP32
mod esp32_event_queue;
#[cfg(target_arch = "xtensa")]
pub use esp32_event_queue::FreeRtosEventQueue;

pub fn start_event_loop(
    queue: Arc<dyn EventQueue>,
    handle_event: Arc<dyn Fn(ModuleEvent) + Send + Sync>,
) {
    #[cfg(feature = "xtensa")]
    {
        Thread::spawn(move || {
            while let Some(event) = queue.receive() {
                handle_event(event);
            }
        })
        .expect("Failed to spawn ESP32 event loop thread");
    }

    #[cfg(not(feature = "xtensa"))]
    {
        std::thread::spawn(move || {
            while let Some(event) = queue.receive() {
                handle_event(event);
            }
        });
    }
}
