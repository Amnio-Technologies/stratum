use crossbeam::channel::{unbounded, Receiver, Sender};

use crate::modules::system_controller::ModuleEvent;

use super::EventQueue;

pub struct MpscEventQueue {
    sender: Sender<ModuleEvent>,
    receiver: Receiver<ModuleEvent>,
}

impl MpscEventQueue {
    pub fn new() -> Self {
        let (tx, rx) = unbounded(); // Crossbeam channel
        Self {
            sender: tx,
            receiver: rx,
        }
    }
}

impl EventQueue for MpscEventQueue {
    fn send(&self, event: ModuleEvent) {
        let _ = self.sender.send(event);
    }

    fn receive(&self) -> Option<ModuleEvent> {
        self.receiver.try_recv().ok()
    }
}
