use egui_ltreeview::TreeViewState;
use std::{
    collections::HashMap,
    ffi::CStr,
    os::raw::c_void,
    sync::{Arc, Mutex},
};
use stratum_ui_common::stratum_ui_ffi::{
    self, export_tree, register_tree_send_callback, FlatNode as FlatNodeRaw,
};

/// Intermediate, cleaned‐up version of the C `FlatNode`.
#[derive(Debug, Clone)]
struct FlatNode {
    ptr: usize,
    parent_ptr: usize,
    class_name: String,
    x: i16,
    y: i16,
    w: i16,
    h: i16,
    hidden: bool,
    debug_id: usize,
}

/// A real tree node: owns its children.
#[derive(Debug, Clone)]
pub struct TreeNode {
    pub ptr: usize,
    pub class_name: String,
    pub x: i16,
    pub y: i16,
    pub w: i16,
    pub h: i16,
    pub hidden: bool,
    pub debug_id: usize,
    pub children: Vec<TreeNode>,
}

/// Manages a single‐root tree snapshot and FFI callback setup.
pub struct TreeManager {
    root: Option<TreeNode>,
    pub selected_obj_ptr: Option<usize>,
}

pub type SharedTreeManager = Arc<Mutex<TreeManager>>;

impl TreeManager {
    /// Construct, register the callback, and return an Arc handle.
    pub fn new() -> SharedTreeManager {
        let mgr = Arc::new(Mutex::new(TreeManager {
            root: None,
            selected_obj_ptr: None,
        }));
        unsafe {
            register_tree_send_callback(Some(tree_send_cb), Arc::as_ptr(&mgr) as *mut c_void);
        }
        mgr
    }

    /// Re-register after a hot reload, if needed.
    pub fn bind_ffi_callback(manager: SharedTreeManager) {
        unsafe {
            register_tree_send_callback(Some(tree_send_cb), Arc::as_ptr(&manager) as *mut c_void);
        }
    }

    /// Trigger C to export the tree, then take ownership of the single root.
    pub fn update_and_take_root(manager: &SharedTreeManager) -> Option<TreeNode> {
        unsafe { export_tree() };
        let guard = manager.lock().unwrap();
        guard.root.clone()
    }

    pub fn request_obj_at_point(manager: &SharedTreeManager, x: usize, y: usize) {
        let mut guard = manager.lock().unwrap();

        unsafe {
            let ptr = stratum_ui_ffi::lvgl_obj_at_point(x as i32, y as i32) as *const c_void;
            guard.selected_obj_ptr = if ptr.is_null() {
                None
            } else {
                Some(ptr as usize)
            };
        }
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
    let mgr_ptr = user_data as *const Mutex<TreeManager>;
    Arc::increment_strong_count(mgr_ptr);
    let mgr: SharedTreeManager = Arc::from_raw(mgr_ptr);

    // Build a Vec<FlatNode> from the raw C array
    let slice = std::slice::from_raw_parts(raw_nodes, count);
    let mut flat = Vec::with_capacity(count);
    for raw in slice {
        let cname = if raw.class_name.is_null() {
            "<null>".to_string()
        } else {
            CStr::from_ptr(raw.class_name)
                .to_string_lossy()
                .into_owned()
        };
        flat.push(FlatNode {
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

    // Convert the flat list into a single‐root TreeNode
    let tree = build_tree(&flat);
    dbg!("hey yall");
    // Store it
    {
        let mut guard = mgr.lock().unwrap();
        guard.root = tree;
    }
    dbg!("hey yall but here now");

    // Drop the temporary Arc; the original Arc still lives elsewhere
    // mgr is dropped here
}

/// Build a single TreeNode root (if any) from the flat list.
fn build_tree(flat: &[FlatNode]) -> Option<TreeNode> {
    // 1) index by ptr
    let mut by_ptr = HashMap::with_capacity(flat.len());
    for node in flat {
        by_ptr.insert(node.ptr, node);
    }

    // 2) group children ptrs under each parent_ptr
    let mut kids = HashMap::<usize, Vec<_>>::with_capacity(flat.len());
    for node in flat {
        kids.entry(node.parent_ptr).or_default().push(node.ptr);
    }

    // 3) recursive constructor
    fn make_node(
        ptr: usize,
        by_ptr: &HashMap<usize, &FlatNode>,
        kids: &HashMap<usize, Vec<usize>>,
    ) -> TreeNode {
        let n = by_ptr.get(&ptr).expect("node pointer must exist");
        let mut tn = TreeNode {
            ptr: n.ptr,
            class_name: n.class_name.clone(),
            x: n.x,
            y: n.y,
            w: n.w,
            h: n.h,
            hidden: n.hidden,
            debug_id: n.debug_id,
            children: Vec::new(),
        };
        if let Some(cptrs) = kids.get(&ptr) {
            tn.children = cptrs
                .iter()
                .map(|&cptr| make_node(cptr, by_ptr, kids))
                .collect();
        }
        tn
    }

    // 4) find the single top-level ptr (parent_ptr == 0)
    kids.get(&0)
        .and_then(|roots| roots.first().copied())
        .map(|root_ptr| make_node(root_ptr, &by_ptr, &kids))
}
