use amnio_common::ui_logging::LogLevel;
use log::error;
use std::{
    any::Any,
    collections::{HashMap, VecDeque},
    sync::{Arc, Mutex},
};
use uom::si::f64::{ElectricCurrent, ElectricPotential, ThermodynamicTemperature};

use crate::events::{create_event_queue, start_event_loop, EventQueue};

/// Log entry struct
#[derive(Debug, Clone)]
pub struct LogEntry {
    pub level: LogLevel,
    pub message: String,
}

#[derive(Debug, Clone)]
pub enum CriticalEvent {
    OverVoltage(ElectricPotential),  // e.g., Voltage exceeds safety limits
    UnderVoltage(ElectricPotential), // Voltage drops below safe level
    OverCurrent(ElectricCurrent),    // e.g., Excessive current detected
    OverTemperature(ThermodynamicTemperature), // e.g., High temperature warning
    ModuleFailure(String),           // e.g., Generic module failure with description
}

#[derive(Debug, Clone)]
pub enum ModuleEvent {
    Critical(CriticalEvent),
    Warning(String),
    Info(String),
    ModuleEvent {
        module_id: u16,
        event: Arc<dyn Any + Send + Sync>,
    },
}

/// SystemController manages modules and logs
pub struct SystemController {
    event_log: Arc<Mutex<VecDeque<String>>>,
    module_logs: Arc<Mutex<HashMap<u16, VecDeque<LogEntry>>>>,
    event_queue: Arc<dyn EventQueue>,
}

impl SystemController {
    pub fn new() -> Arc<Self> {
        let queue: Arc<dyn EventQueue> = Arc::new(create_event_queue());

        let controller = Arc::new(Self {
            event_log: Arc::new(Mutex::new(VecDeque::with_capacity(10_000))),
            module_logs: Arc::new(Mutex::new(HashMap::new())),
            event_queue: Arc::clone(&queue),
        });

        let controller_clone = Arc::clone(&controller); // Clone controller for the closure

        start_event_loop(
            queue,
            Arc::new(move |event| controller_clone.handle_event(event)),
        );

        controller
    }

    pub fn handle_event(&self, event: ModuleEvent) {
        if let Ok(mut log) = self.event_log.lock() {
            log.push_back(format!("{:?}", event));
        } else {
            error!("Failed to acquire event_log lock (mutex poisoned)");
        }
    }

    pub fn emit_event(&self, event: ModuleEvent) {
        self.event_queue.send(event);
    }

    pub fn log_module_event(&self, module_id: u16, entry: LogEntry) {
        if let Ok(mut logs) = self.module_logs.lock() {
            let module_log = logs
                .entry(module_id)
                .or_insert_with(|| VecDeque::with_capacity(100));
            if module_log.len() >= 100 {
                module_log.pop_front();
            }
            module_log.push_back(entry);
        } else {
            error!("Failed to acquire module_logs lock (mutex poisoned)");
        }
    }

    pub fn get_filtered_logs(&self, level: LogLevel) -> Vec<String> {
        if let Ok(logs) = self.event_log.lock() {
            logs.iter()
                .filter(|entry| entry.contains(&format!("{:?}", level)))
                .cloned()
                .collect()
        } else {
            error!("Failed to acquire event_log lock (mutex poisoned)");
            Vec::new()
        }
    }
}
