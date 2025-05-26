pub mod lvgl_backend;
pub mod ui_logging;

pub mod amnio_bindings {
    #![allow(
        non_camel_case_types,
        non_snake_case,
        non_upper_case_globals,
        dead_code
    )]

    // Raw bindgen declarations (always compiled)
    mod raw {
        #[cfg(windows)]
        include!(concat!(env!("OUT_DIR"), "\\bindings.rs"));
        #[cfg(unix)]
        include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
    }

    // Dynamic loader + API table (desktop hot-reload only)
    #[cfg(feature = "desktop")]
    mod dynamic_overrides {
        pub use super::raw::*; // bring raw types, constants into scope

        // Generated loader and Api struct
        mod internal_api {
            #[cfg(windows)]
            include!(concat!(env!("OUT_DIR"), "\\internal_api.rs"));
            #[cfg(unix)]
            include!(concat!(env!("OUT_DIR"), "/internal_api.rs"));
        }

        // Generated wrapper functions that dispatch via the Api table
        #[cfg(windows)]
        include!(concat!(env!("OUT_DIR"), "\\dynamic_exports.rs"));
        #[cfg(unix)]
        include!(concat!(env!("OUT_DIR"), "/dynamic_exports.rs"));

        // Expose only the loader function from internal_api
        pub use internal_api::init_dynamic_bindings;
    }

    // Public exports: choose dynamic overrides on desktop, else raw externs
    #[cfg(feature = "desktop")]
    pub use dynamic_overrides::*;
    #[cfg(not(feature = "desktop"))]
    pub use raw::*;
}
