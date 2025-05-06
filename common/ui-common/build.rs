use std::{env, fs, path::PathBuf};

fn main() {
    // Directories
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let bindings_src_dir = manifest_dir.join("bindings");
    let bindings_src_file = bindings_src_dir.join("bindings.rs");
    let bindings_out_file = out_dir.join("bindings.rs");

    // Host vs Target
    let host = env::var("HOST").unwrap();
    let target = env::var("TARGET").unwrap();

    if target == host {
        // Native build: run bindgen
        let amnio_root = manifest_dir.parent().and_then(|p| p.parent()).unwrap();
        let inc_dir_amnio = amnio_root.join("stratum-ui").join("include");
        let inc_dir_lvgl = inc_dir_amnio.join("lvgl");
        let header_to_bind = inc_dir_amnio.join("stratum_ui.h");
        let fallback_clang_include = "/usr/lib/clang/15/include"; // adjust if needed

        let bindings = bindgen::Builder::default()
            .layout_tests(false) // disable layout tests for embedded target
            .header(header_to_bind.to_string_lossy())
            .clang_args(&[
                format!("-I{}", inc_dir_amnio.display()),
                format!("-I{}", inc_dir_lvgl.display()),
                format!("-isystem{}", fallback_clang_include),
            ])
            .raw_line("#[allow(dead_code)]")
            .generate()
            .expect("Unable to generate bindings from stratum_ui.h");

        // Write to OUT_DIR
        bindings
            .write_to_file(&bindings_out_file)
            .expect("Couldn't write bindings to OUT_DIR");

        // Also update committed fallback
        fs::create_dir_all(&bindings_src_dir).unwrap();
        fs::copy(&bindings_out_file, &bindings_src_file)
            .expect("Failed to update committed bindings.rs");

        println!("cargo:rerun-if-changed={}", bindings_src_file.display());
    } else {
        // Cross-compile: copy pre-generated bindings into OUT_DIR
        fs::create_dir_all(&out_dir).unwrap();
        fs::copy(&bindings_src_file, &bindings_out_file)
            .expect("Failed to copy pre-generated bindings.rs into OUT_DIR");
        println!("cargo:warning=Skipping bindgen: cross-compiling for target '{target}'");
        println!("cargo:rerun-if-changed={}", bindings_src_file.display());
    }

    // ----- Link static stratum-ui library -----
    // Detect target kind from metadata
    let cargo_toml_path = manifest_dir.join("Cargo.toml");
    let target_kind = fs::read_to_string(&cargo_toml_path)
        .ok()
        .and_then(|s| toml::from_str::<toml::Value>(&s).ok())
        .and_then(|doc| {
            doc.get("package")?
                .get("metadata")?
                .get("stratum-ui-target")?
                .as_str()
                .map(|s| s.to_string())
        })
        .unwrap_or_else(|| "desktop".to_string());

    let amnio_root = manifest_dir.parent().and_then(|p| p.parent()).unwrap();
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
}
