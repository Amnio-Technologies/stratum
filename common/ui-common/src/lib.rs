pub mod lvgl_backend;
pub mod ui_logging;

pub mod amnio_bindings {
    #![allow(
        non_camel_case_types,
        non_snake_case,
        non_upper_case_globals,
        dead_code
    )]

    #[cfg(windows)]
    include!(concat!(env!("OUT_DIR"), "\\bindings.rs"));

    #[cfg(unix)]
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}
