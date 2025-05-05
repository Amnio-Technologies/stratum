use std::env;
use std::path::PathBuf;

fn main() {
    let cwd = env::current_dir().unwrap();
    let amnio_root = cwd.parent().and_then(|p| p.parent()).unwrap();

    let stratum_ui_build = amnio_root.join("stratum-ui").join("build");
    let lib_path = stratum_ui_build.join("libstratum-ui.a");

    // Ensure the static library exists
    if !lib_path.exists() {
        panic!("❌ Static library libstratum-ui.a not found! Did you build stratum-ui?");
    }

    // Link against the static library
    println!(
        "cargo:rustc-link-search=native={}",
        stratum_ui_build.display()
    );
    println!("cargo:rustc-link-lib=static=stratum-ui");

    // Ensure Cargo rebuilds if the library changes
    println!("cargo:rerun-if-changed={}", lib_path.display());

    let inc_dir_amnio = amnio_root.join("stratum-ui").join("include");
    let inc_dir_lvgl = inc_dir_amnio.join("lvgl");
    let header_to_bind = inc_dir_amnio.join("stratum_ui.h");

    let bindings = bindgen::Builder::default()
        .header(header_to_bind.to_string_lossy())
        .clang_args(&[
            format!("-I{}", inc_dir_amnio.display()),
            format!("-I{}", inc_dir_lvgl.display()),
        ])
        .raw_line("#[allow(dead_code)]")
        .generate()
        .expect("Unable to generate bindings from stratum_ui.h");

    let binding_out_path = PathBuf::from(env::var("OUT_DIR").unwrap()).join("bindings.rs");
    bindings
        .write_to_file(&binding_out_path)
        .expect("Couldn't write bindings");

    println!(
        "cargo:rustc-env=BINDINGS_FILE={}",
        binding_out_path.display()
    );
}
