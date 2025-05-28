use crate::stratum_ui_ffi::{self, register_ui_log_callback};
use std::ffi::CStr;
use std::os::raw::{c_char, c_void};
use std::sync::{Arc, Mutex};

/// Mirror the C enum exactly.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogLevel {
    Trace = 0,
    Debug = 1,
    Info = 2,
    Warn = 3,
    Error = 4,
}

/// Holds all logger state, including a cap on retained entries.
pub struct UiLogger {
    logs: Mutex<Vec<String>>,
    max_logs: usize,
}

impl UiLogger {
    /// Create, register once, and return the Arc.
    pub fn new(max_logs: usize) -> Arc<Self> {
        let logger = Arc::new(UiLogger {
            logs: Mutex::new(Vec::with_capacity(max_logs)),
            max_logs,
        });

        // Register the callback now (initial load)
        unsafe {
            register_ui_log_callback(Some(ui_log_callback), Arc::as_ptr(&logger) as *mut c_void);
        }

        logger
    }

    /// Re‐register after each hot reload.
    pub fn bind_callback(self: Arc<Self>) {
        unsafe {
            register_ui_log_callback(Some(ui_log_callback), Arc::as_ptr(&self) as *mut c_void);
        }
    }

    pub fn take_logs(&self) -> Vec<String> {
        let mut guard = self.logs.lock().unwrap();
        std::mem::take(&mut *guard)
    }
}

unsafe extern "C" fn ui_log_callback(
    user_data: *mut c_void,
    level: stratum_ui_ffi::LogLevel,
    msg: *const c_char,
) {
    if user_data.is_null() || msg.is_null() {
        return;
    }

    let ptr = user_data as *const UiLogger;
    Arc::increment_strong_count(ptr);

    let logger: Arc<UiLogger> = Arc::from_raw(ptr);

    let s = CStr::from_ptr(msg).to_string_lossy().into_owned();

    {
        let mut logs = logger.logs.lock().unwrap();
        if logs.len() >= logger.max_logs {
            logs.remove(0);
        }
        logs.push(format!("[{:?}] {}", level, s));
    }
}
