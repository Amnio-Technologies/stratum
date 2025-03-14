use anyhow::Result;
use std::{
    fmt::{self, Debug},
    sync::Arc,
};

use super::system_controller::SystemController;

#[derive(Debug, Clone)]
pub struct ModuleMetadata {
    pub id: u16,                 // Unique identifier for the module
    pub name: String,            // Human-readable module name
    pub module_kind: ModuleKind, // Categorized module type
    pub version: String,         // Firmware version
}

#[derive(Debug, Clone)]
pub enum ModuleKind {
    Battery,
    WaveformGenerator,
    SolderingUnit,
    Unknown,
}

impl fmt::Display for ModuleKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ModuleKind::Battery => write!(f, "Battery"),
            ModuleKind::SolderingUnit => write!(f, "Soldering Unit"),
            ModuleKind::WaveformGenerator => write!(f, "Waveform Generator"),
            ModuleKind::Unknown => write!(f, "Unknown"),
        }
    }
}

pub trait ModuleCommand {
    type Response;

    fn as_any(&self) -> &dyn std::any::Any;
}

use thiserror::Error;

#[derive(Debug, Error)]
pub enum ModuleError {
    #[error("Invalid command: {0}")]
    InvalidCommand(String),

    #[error("Failed to downcast response")]
    DowncastFailure,

    #[error("Hardware failure: {0}")]
    HardwareFailure(String),

    #[error("Initialization error")]
    InitializationError,

    #[error("Unknown error")]
    Unknown,
}

type ModuleCommandExecutionResponse = Result<Box<dyn std::any::Any>, ModuleError>;

pub trait Module {
    type ModuleCommand;
    type ModuleStatus;

    fn metadata(&self) -> ModuleMetadata;

    fn process_command(&mut self, command: Self::ModuleCommand) -> ModuleCommandExecutionResponse;

    fn status(&self) -> Self::ModuleStatus;

    fn initialize(&mut self, system_controller: Arc<SystemController>) -> Result<(), ModuleError>;
}

/// A macro to define module commands and their corresponding structures.
///
/// This macro generates an enum representing all possible commands for a given module,
/// along with structs implementing the `ModuleCommand` trait for each command.
///
/// # Example
/// ```
/// def_module_commands! {
///     my_module {
///         Foo(arg1: u32, arg2: String) -> bool;
///         Bar(arg: i32) -> ();
///         Baz() -> ();
///     }
/// }
/// ```
///
/// This expands to:
/// ```rust
/// pub mod my_module {
///     use crate::modules::module::ModuleCommand;
///
///     /// Enum representing all commands for the module.
///     #[derive(Debug)]
///     pub enum MyModule {
///         Foo { arg1: u32, arg2: String },
///         Bar { arg: i32 },
///         Baz { },
///     }
///
///     /// Struct representing the `Foo` command.
///     pub struct Foo;
///
///     impl ModuleCommand for Foo {
///         type Response = bool;
///
///         fn as_any(&self) -> &dyn std::any::Any {
///             self
///         }
///     }
///
///     /// Struct representing the `Bar` command.
///     pub struct Bar;
///
///     impl ModuleCommand for Bar {
///         type Response = ();
///
///         fn as_any(&self) -> &dyn std::any::Any {
///             self
///         }
///     }
///
///     /// Struct representing the `Baz` command (unit struct).
///     pub struct Baz;
///
///     impl ModuleCommand for Baz {
///         type Response = ();
///
///         fn as_any(&self) -> &dyn std::any::Any {
///             self
///         }
///     }
/// }
/// ```
///
/// This ensures that each command in the module has a corresponding struct that implements
/// the `ModuleCommand` trait, defining the associated response type for each command.
/// If a command has no arguments, it is treated as a unit struct variant instead of an empty struct.
#[macro_export]
macro_rules! def_module_commands {
    ($(#[$meta:meta])* $module_name:ident {
        $( $cmd_name:ident ($($arg_name:ident : $arg_ty:ty),* ) $(-> $ret_ty:ty)?; )*
    }) => {
        $(#[$meta])*
        pub mod $module_name {
            use crate::modules::module::ModuleCommand;

            paste::paste! {
                /// Enum representing all commands for the module.
                #[derive(Debug)]
                pub enum [<$module_name:camel>] {
                    $( $cmd_name { $( $arg_name: $arg_ty ),* } ),*
                }
            }

            $(
                /// Struct representing the `$cmd_name` command.
                pub struct $cmd_name;

                impl ModuleCommand for $cmd_name {
                    type Response = $( $ret_ty )?;

                    fn as_any(&self)  -> &dyn std::any::Any {
                        self
                    }
                }
            )*
        }
    };
}

/// A macro that performs a type-enforced match for a module command enum.
///
/// This macro ensures that the response type of each command variant is correctly inferred.
/// It immediately evaluates the provided closure body and returns the result as a boxed `Any` type.
///
/// # Example
/// ```
/// command_match!(
///     command_enum,
///     parent_module,
///     my_module,
///     Foo { arg1, arg2 } => some_function(arg1, arg2),
///     Bar { arg } => another_function(arg),
/// )
/// ```
#[macro_export]
macro_rules! command_match {
    ($cmd_enum:expr, $parent:path, $module_name:ident, $( $variant:ident { $( $arg:ident ),* } => $body:expr ),* $(,)?) => {{
        paste::paste! {
            match $cmd_enum {
                $(
                    $module_name::$variant { $( $arg ),* } => {
                        let result = (|| -> <$parent::[<$module_name:snake>]::$variant as crate::modules::module::ModuleCommand>::Response {
                            $body
                        })(); // Invoke the closure immediately

                        Ok(Box::new(result) as Box<dyn std::any::Any>)
                    }
                ),*
            }
        }
    }};
}
