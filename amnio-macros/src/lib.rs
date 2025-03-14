use proc_macro::TokenStream;
use quote::quote;
use syn::{ExprCall, parse_macro_input};

/// Implements `execute_command!` to allow calling as `module.execute_command!(path::Command {})`
#[proc_macro]
pub fn execute_command(input: TokenStream) -> TokenStream {
    let input_expr = parse_macro_input!(input as ExprCall); // Parse the macro input as a function call

    let func = &input_expr.func;
    let args = &input_expr.args;

    let expanded = quote! {
        {
            use std::any::Any;
            use amnio_firmware::modules::module::{ModuleCommand, ModuleError};

            let command_instance = #func #args;

            type ResponseType = <#func as ModuleCommand>::Response;

            self.process_command(command_instance)
                .and_then(|boxed_result| {
                    boxed_result
                        .downcast::<ResponseType>()
                        .map(|boxed| *boxed)
                        .map_err(|_| ModuleError::DowncastFailure)
                })
        }
    };

    expanded.into()
}
