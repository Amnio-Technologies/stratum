use std::env;
use std::path::PathBuf;

fn main() {
    let cwd = env::current_dir().unwrap();
    let amnio_root = cwd.parent().and_then(|p| p.parent()).unwrap();

    let amnio_ui_build = amnio_root.join("ui").join("amnio-ui").join("build");
    let lib_path = amnio_ui_build.join("libamnio-ui.a"); // Use static library now!

    // Ensure the static library exists
    if !lib_path.exists() {
        panic!("❌ Static library libamnio-ui.a not found! Did you build amnio-ui?");
    }

    // Link against the static library
    println!(
        "cargo:rustc-link-search=native={}",
        amnio_ui_build.display()
    );
    println!("cargo:rustc-link-lib=static=amnio-ui"); // Change `dylib` to `static` ✅

    // Ensure Cargo rebuilds if the library changes
    println!("cargo:rerun-if-changed={}", lib_path.display());

    let inc_dir_amnio = amnio_root.join("ui").join("amnio-ui").join("include");
    let inc_dir_lvgl = inc_dir_amnio.join("lvgl");
    let header_to_bind = inc_dir_amnio.join("amnio_ui.h");

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
