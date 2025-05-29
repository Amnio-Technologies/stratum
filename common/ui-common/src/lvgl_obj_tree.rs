use crate::stratum_ui_ffi::FlatNode;

#[no_mangle]
pub extern "C" fn send_tree(nodes: *const FlatNode, count: usize) {
    println!("{nodes:?}, {count}");
}
