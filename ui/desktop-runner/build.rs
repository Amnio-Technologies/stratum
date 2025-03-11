extern crate cc;
use std::path::PathBuf;

fn main() {
    let amnio_ui_path = PathBuf::from("../amnio-ui/src/amnio_ui.c");

    cc::Build::new()
        .file(amnio_ui_path)
        .include("../amnio-ui/include") // ✅ Include amnio_ui.h
        .include("../amnio-ui/include/lvgl") // ✅ Ensure LVGL headers are found
        .compile("amnio_ui");

    println!("cargo:rerun-if-changed=../amnio-ui/src/amnio_ui.c");
}
