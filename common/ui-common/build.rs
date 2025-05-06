use std::{
    env, fs,
    path::{Path, PathBuf},
};

fn main() {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let amnio_root = manifest_dir.parent().and_then(|p| p.parent()).unwrap();
    println!(
        "cargo:warning=CARGO_MANIFEST_DIR = {}",
        manifest_dir.display()
    );

    // Detect target type from Cargo.toml
    let cargo_toml_path = Path::new(&manifest_dir).join("Cargo.toml");
    let target_kind = fs::read_to_string(&cargo_toml_path)
        .ok()
        .and_then(|toml| toml.parse::<toml::Value>().ok())
        .and_then(|doc| {
            doc.get("package")?
                .get("metadata")?
                .get("stratum-ui-target")?
                .as_str()
                .map(|s| s.to_string())
        })
        .unwrap_or_else(|| "desktop".to_string());

    // stratum-ui/build/<kind>/libstratum-ui.a
    let stratum_ui_build = amnio_root
        .join("stratum-ui")
        .join("build")
        .join(&target_kind);
    let lib_path = stratum_ui_build.join("libstratum-ui.a");

    if !lib_path.exists() {
        panic!(
            "❌ Static library not found for target '{}': {}",
            target_kind,
            lib_path.display()
        );
    }

    println!(
        "cargo:rustc-link-search=native={}",
        stratum_ui_build.display()
    );
    println!("cargo:rustc-link-lib=static=stratum-ui");
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
