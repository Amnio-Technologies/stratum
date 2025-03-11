use std::fs;
use std::path::PathBuf;

fn main() {
    // Ensure the correct DLL name
    let dll_path =
        PathBuf::from("C:/Users/erick/Documents/amnio/ui/amnio-ui/build/libamnio-ui.dll");

    let project_root_debug = PathBuf::from("C:/Users/erick/Documents/amnio/target/debug");

    // Ensure the target/debug directory exists
    fs::create_dir_all(&project_root_debug).expect("Failed to create target/debug directory");

    // Copy DLL to target/debug so it's next to the Rust exe
    let dll_dest = project_root_debug.join("libamnio-ui.dll");
    fs::copy(&dll_path, &dll_dest).expect("Failed to copy amnio-ui.dll to target/debug");

    // Link against the import library (MinGW generates `libamnio-ui.dll.a`)
    println!("cargo:rustc-link-search=native=C:/Users/erick/Documents/amnio/ui/amnio-ui/build");
    println!("cargo:rustc-link-lib=dylib=amnio-ui");

    // Ensure Cargo rebuilds if the DLL changes
    println!("cargo:rerun-if-changed={}", dll_path.to_string_lossy());
}
