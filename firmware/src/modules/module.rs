use super::system_controller::SystemController;
use anyhow::Result;
use std::{
    fmt::{self, Debug},
    sync::Arc,
};
use thiserror::Error;

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

#[derive(Debug, Error)]
pub enum ModuleCommandExecutionError {
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

pub type ModuleCommandExecutionResponse =
    Result<Box<dyn std::any::Any>, ModuleCommandExecutionError>;

pub trait Module {
    type ModuleCommand;
    type ModuleStatus;

    fn metadata(&self) -> ModuleMetadata;

    fn process_command(&mut self, command: Self::ModuleCommand) -> ModuleCommandExecutionResponse;

    fn status(&self) -> Self::ModuleStatus;

    fn initialize(
        &mut self,
        system_controller: Arc<SystemController>,
    ) -> Result<(), ModuleCommandExecutionError>;
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
    ($cmd_enum:expr, $module_name:ident,
        $(
            $variant:ident $( { $( $arg:ident ),* } )? => $body:expr
        ),* $(,)?
    ) => {{
        paste::paste! {
            match $cmd_enum {
                $(
                    $module_name::$variant $( { $( $arg ),* } )? => {
                        let result = (|| -> <crate::modules::commands::[<$module_name:snake>]::$variant as crate::modules::module::ModuleCommand>::Response {
                            $body
                        })(); // Invoke the closure immediately (this is to allow early returns inside the body)

                        Ok(Box::new(result) as Box<dyn std::any::Any>)
                    }
                ),*
            }
        }
    }};
}

pub use amnio_macros::def_module_commands;
pub use amnio_macros::execute_command;
