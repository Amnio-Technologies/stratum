use std::env;
use std::fs;
use std::path::PathBuf;

fn main() {
    let cwd = env::current_dir().unwrap();
    let amnio_root = cwd.parent().and_then(|p| p.parent()).unwrap();
    let target_dir = amnio_root.join("target");

    let amnio_ui_build = amnio_root.join("ui").join("amnio-ui").join("build");
    let dll_path = amnio_ui_build.join("libamnio-ui.dll");
    let project_root_debug = target_dir.join("debug");

    fs::create_dir_all(&project_root_debug).expect("Failed to create target/debug directory");

    // Copy DLL to target/debug so it's next to the Rust exe
    let dll_dest = project_root_debug.join("libamnio-ui.dll");
    fs::copy(&dll_path, &dll_dest).expect("Failed to copy amnio-ui.dll to target/debug");

    // Link against the import library (MinGW generates `libamnio-ui.dll.a`)
    println!(
        "cargo:rustc-link-search=native={}",
        amnio_ui_build.display()
    );
    println!("cargo:rustc-link-lib=dylib=amnio-ui");

    // Ensure Cargo rebuilds if the DLL changes
    println!("cargo:rerun-if-changed={}", dll_path.display());

    let inc_dir_amnio = amnio_root.join("ui").join("amnio-ui").join("include");

    let inc_dir_lvgl = amnio_root
        .join("ui")
        .join("amnio-ui")
        .join("include")
        .join("lvgl");

    let header_to_bind = amnio_root
        .join("ui")
        .join("amnio-ui")
        .join("include")
        .join("amnio_ui.h");

    dbg!(&header_to_bind.clone());
    dbg!(format!("{}", header_to_bind.display()));

    let bindings = bindgen::Builder::default()
        .header(header_to_bind.to_string_lossy())
        // Pass the same -I paths that your CMake uses
        .clang_args(&[
            format!("-I{}", inc_dir_amnio.display()),
            format!("-I{}", inc_dir_lvgl.display()),
        ])
        .generate()
        .expect("Unable to generate bindings from amnio_ui.h");

    let binding_out_path = PathBuf::from(env::var("OUT_DIR").unwrap()).join("bindings.rs");
    bindings
        .write_to_file(&binding_out_path)
        .expect("Couldn't write bindings");

    println!(
        "cargo:rustc-env=BINDINGS_FILE={}",
        binding_out_path.display()
    );
}
