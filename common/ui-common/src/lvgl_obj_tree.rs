use crate::stratum_ui_ffi::{export_tree, register_tree_send_callback, FlatNode as FlatNodeRaw};
use std::ffi::CStr;
use std::os::raw::c_void;
use std::sync::{Arc, Mutex};

/// A cleaned‐up Rust-side version of the C `FlatNode`.
#[derive(Debug, Clone)]
pub struct Node {
    pub ptr: usize,
    pub parent_ptr: usize,
    pub class_name: String,
    pub x: i16,
    pub y: i16,
    pub w: i16,
    pub h: i16,
    pub hidden: bool,
    pub debug_id: usize,
}

/// Manages the latest tree snapshot and handles FFI registration.
pub struct TreeManager {
    /// Protected buffer of the last-received nodes
    pub nodes: Mutex<Vec<Node>>,
}

impl TreeManager {
    /// Create & register the callback; returns an Arc you hold onto.
    pub fn new() -> Arc<Self> {
        let mgr = Arc::new(TreeManager {
            nodes: Mutex::new(Vec::new()),
        });

        // Register the C callback, passing our Arc pointer as user_data
        unsafe {
            register_tree_send_callback(Some(tree_send_cb), Arc::as_ptr(&mgr) as *mut c_void);
        }

        mgr
    }

    /// Re‐register after each hot reload.
    pub fn bind_ffi_callback(self: Arc<Self>) {
        unsafe {
            register_tree_send_callback(Some(tree_send_cb), Arc::as_ptr(&self) as *mut c_void);
        }
    }

    /// Take the last tree snapshot out
    pub fn take_nodes(&self) -> Vec<Node> {
        unsafe {
            export_tree();
        }

        std::mem::take(&mut *self.nodes.lock().unwrap())
    }
}

unsafe extern "C" fn tree_send_cb(
    user_data: *mut c_void,
    raw_nodes: *const FlatNodeRaw,
    count: usize,
) {
    if user_data.is_null() || raw_nodes.is_null() {
        return;
    }

    // Reconstruct our Arc<TreeManager> without dropping the real one
    let ptr = user_data as *const TreeManager;
    Arc::increment_strong_count(ptr);
    let mgr: Arc<TreeManager> = Arc::from_raw(ptr);

    // Turn the C array into a Rust slice
    let slice = std::slice::from_raw_parts(raw_nodes, count);

    // Lock & rebuild the Vec<Node>
    let mut guard = mgr.nodes.lock().unwrap();
    guard.clear();
    guard.reserve(count);

    for raw in slice {
        // SAFE: class_name is always a NUL-terminated C string
        let cname = if raw.class_name.is_null() {
            "<null>".to_string()
        } else {
            CStr::from_ptr(raw.class_name)
                .to_string_lossy()
                .into_owned()
        };

        guard.push(Node {
            ptr: raw.ptr,
            parent_ptr: raw.parent_ptr,
            class_name: cname,
            x: raw.x,
            y: raw.y,
            w: raw.w,
            h: raw.h,
            hidden: raw.hidden,
            debug_id: raw.debug_id as usize,
        });
    }

    // Drop the Arc we created via from_raw, balancing the increment
    // (but leaving the original Arc alive)
    // mgr goes out of scope here
}
