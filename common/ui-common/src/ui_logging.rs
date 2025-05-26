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
    /// Creates the logger, registers the C callback, and returns a shared handle.
    pub fn new(max_logs: usize) -> Arc<Self> {
        let logger = Arc::new(UiLogger {
            logs: Mutex::new(Vec::with_capacity(max_logs)),
            max_logs,
        });

        // Pass an Arc pointer as `user_data`; callback recovers it.
        let user_data = Arc::into_raw(logger.clone()) as *mut c_void;
        unsafe {
            register_ui_log_callback(Some(ui_log_callback), user_data);
        }

        logger
    }

    /// Grab a snapshot of all logs so far and clear the buffer.
    pub fn take_logs(&self) -> Vec<String> {
        let mut guard = self.logs.lock().unwrap();
        std::mem::take(&mut *guard)
    }
}

/// Function pointer C will call.
/// Reconstructs the Arc, bumps the refcount, then forgets the original.
unsafe extern "C" fn ui_log_callback(
    user_data: *mut c_void,
    level: stratum_ui_ffi::LogLevel,
    msg: *const c_char,
) {
    if user_data.is_null() || msg.is_null() {
        return;
    }

    // Recover Arc<UiLogger>
    let arc = unsafe { Arc::from_raw(user_data as *const UiLogger) };
    let logger = arc.clone(); // bump reference count
    std::mem::forget(arc); // avoid dropping the original

    // Convert C string
    let s = unsafe { CStr::from_ptr(msg) }
        .to_string_lossy()
        .into_owned();

    // Push into buffer with rotation
    let mut logs = logger.logs.lock().unwrap();
    if logs.len() >= logger.max_logs {
        logs.remove(0);
    }
    logs.push(format!("[{:?}] {}", level, s));
}
