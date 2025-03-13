use crate::modules::system_controller::ModuleEvent;

#[cfg(target_arch = "xtensa")] // Runs only on ESP32
pub struct FreeRtosEventQueue {
    queue: QueueHandle_t,
}

#[cfg(target_arch = "xtensa")]
impl FreeRtosEventQueue {
    pub fn new() -> Self {
        let queue = unsafe { xQueueCreate(10, std::mem::size_of::<ModuleEvent>() as u32) };
        Self { queue }
    }
}

#[cfg(target_arch = "xtensa")]
impl EventQueue for FreeRtosEventQueue {
    fn send(&self, event: ModuleEvent) {
        unsafe {
            xQueueSend(self.queue, &event as *const _ as *const c_void, 0);
        }
    }

    fn receive(&self) -> Option<ModuleEvent> {
        let mut event = ModuleEvent::Info("".to_string());
        let result = unsafe {
            xQueueReceive(
                self.queue,
                &mut event as *mut _ as *mut c_void,
                portMAX_DELAY,
            )
        };
        if result != 0 {
            Some(event)
        } else {
            None
        }
    }
}

#[cfg(target_arch = "xtensa")]
unsafe impl Send for FreeRtosEventQueue {}

#[cfg(target_arch = "xtensa")]
unsafe impl Sync for FreeRtosEventQueue {}
