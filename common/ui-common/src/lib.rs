pub mod lvgl_backend;
pub mod ui_logging;

/* -------------------------------------------------------------------------- */
/*  Public FFI surface  ───────────────────────────────────────────────────── */
pub mod amnio_bindings {
    #![allow(
        non_camel_case_types,
        non_snake_case,
        non_upper_case_globals,
        dead_code
    )]

    /* raw bindgen output (always present) */
    #[cfg(windows)]
    include!(concat!(env!("OUT_DIR"), "\\bindings.rs"));
    #[cfg(unix)]
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

    #[cfg(feature = "desktop")]
    mod dynamic_overrides {
        /* bring raw types / consts into scope for the wrappers */
        use super::*;

        /* loader + Api struct (generated) */
        mod internal_api {
            #[cfg(windows)]
            include!(concat!(env!("OUT_DIR"), "\\internal_api.rs"));
            #[cfg(unix)]
            include!(concat!(env!("OUT_DIR"), "/internal_api.rs"));
        }

        /* shadow functions with wrappers that call API.read()… */
        #[cfg(windows)]
        include!(concat!(env!("OUT_DIR"), "\\dynamic_exports.rs"));
        #[cfg(unix)]
        include!(concat!(env!("OUT_DIR"), "/dynamic_exports.rs"));

        pub use internal_api::init_dynamic_bindings;
    }

    #[cfg(feature = "desktop")]
    pub use dynamic_overrides::*;
}
