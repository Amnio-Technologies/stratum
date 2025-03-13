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

    fn box_return<C: crate::modules::module::ModuleCommand + 'static>(
        return_val: C::Response,
    ) -> ModuleCommandExecutionResponse {
        Ok(Box::new(return_val))
    }

    fn process_command<C>(&mut self, command: C) -> ModuleCommandExecutionResponse
    where
        C: crate::modules::module::ModuleCommand,
        C: Into<Self::ModuleCommand>;

    fn execute_command<M, C>(module: &mut M, command: C) -> Result<C::Response, ModuleError>
    where
        M: Module,
        C: ModuleCommand + Into<M::ModuleCommand>,
        C::Response: 'static,
    {
        let result_any = module.process_command(command)?;

        result_any
            .downcast::<C::Response>()
            .map(|boxed| *boxed)
            .map_err(|_| ModuleError::DowncastFailure)
    }

    fn status(&self) -> Self::ModuleStatus;

    fn initialize(&mut self, system_controller: Arc<SystemController>) -> Result<(), ModuleError>;
}

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
                    $( $cmd_name($cmd_name) ),*
                }

                // Automatically implement From<T> for each command struct
                $(
                    impl From<$cmd_name> for [<$module_name:camel>] {
                        fn from(cmd: $cmd_name) -> Self {
                            Self::$cmd_name(cmd)
                        }
                    }
                )*
            }

            $(
                #[derive(Debug)]
                /// Struct representing the `$cmd_name` command.
                pub struct $cmd_name {
                    $(pub $arg_name: $arg_ty),*
                }

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
