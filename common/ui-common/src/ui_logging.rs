use std::ffi::CStr;
use std::sync::mpsc::{self, Sender};
use std::sync::{Arc, Mutex, OnceLock};
use std::thread;

use log::{error, info, warn};

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogLevel {
    Trace = 0,
    Debug = 1,
    Info = 2,
    Warn = 3,
    Error = 4,
}

const MAX_LOGS: usize = 10_000;

lazy_static::lazy_static! {
    pub static ref UI_LOGS: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(vec![]));
}

static UI_LOG_SENDER: OnceLock<Sender<String>> = OnceLock::new();

/// Background thread to move logs from C into UI_LOGS safely
pub fn start_ui_log_worker() {
    info!("UI log worker initialized");

    let (tx, rx) = mpsc::channel();
    let result = UI_LOG_SENDER.set(tx);

    match result {
        Ok(_) => info!("UI_LOG_SENDER set successfully"),
        Err(_) => warn!("UI_LOG_SENDER was already set!"),
    }

    thread::spawn(move || {
        info!("Log worker thread started");

        while let Ok(log) = rx.recv() {
            if let Ok(mut logs) = UI_LOGS.lock() {
                if logs.len() >= MAX_LOGS {
                    logs.remove(0); // Remove the oldest log to maintain limit
                }
                logs.push(log);
            }
        }

        info!("Log worker thread shutting down");
    });
}

/// Expose logging function to C (non-blocking)
#[no_mangle]
pub extern "C" fn ui_log(level: LogLevel, msg: *const std::ffi::c_char) {
    if msg.is_null() {
        return;
    }

    let msg = unsafe { CStr::from_ptr(msg) }.to_string_lossy().to_string();
    let formatted_msg = format!("[{:?}] {}", level, msg);

    match UI_LOG_SENDER.get() {
        Some(sender) => {
            let _ = sender.send(formatted_msg);
        }
        None => {
            error!("UI_LOG_SENDER is still None! Did start_ui_log_worker() run?");
        }
    }
}
