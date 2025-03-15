use convert_case::{Case, Casing};
use execute_command::CommandInput;
use proc_macro::TokenStream;
use syn::parse_macro_input;
mod execute_command;
mod generate_module_commands;
use quote::{format_ident, quote};

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
#[proc_macro]
pub fn def_module_commands(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as generate_module_commands::ModuleCommandDef);
    generate_module_commands::generate_module_commands(input)
}
/// A procedural macro for executing module commands while resolving the correct response type at compile time.
///
/// This macro generates the correct function calls and type resolution for a given module command, ensuring that:
/// - The correct module and command types are used.
/// - The response type is properly inferred and downcasted from `Box<dyn Any>`.
/// - Qualified command variants (e.g., `BatteryCommand::SetOutput { state: false }`) are fully supported.
///
/// ## **Usage**
///
/// ```rust
/// execute_command!(dummy_module: BatteryCommand, BatteryCommand::SetOutput { state: false });
/// ```
///
/// ## **Expands To**
/// ```rust
/// {
///     type CommandStruct = amnio_firmware::modules::commands::battery_command::SetOutput;
///     type ResponseType = <CommandStruct as ModuleCommand>::Response;
///
///     let command = BatteryCommand::SetOutput { state: false };
///     let result: Result<Box<dyn Any>, ModuleError> = dummy_module.process_command(command);
///     let final_result: Result<ResponseType, ModuleError> = result.map(|boxed| *boxed.downcast::<ResponseType>().unwrap());
///
///     final_result
/// }
/// ```
///
/// ## **Macro Arguments**
/// - **`module`** → The instance of the module (e.g., `dummy_module`).
/// - **`module_command_type`** → The command enum type (e.g., `BatteryCommand`).
/// - **`command_variant`** → The fully qualified command with arguments (e.g., `BatteryCommand::SetOutput { state: false }`).
///
/// ## **How It Works**
/// 1. **Converts the command module to `snake_case`** (e.g., `BatteryCommand` → `battery_command`).
/// 2. **Derives the fully qualified struct path** (e.g., `commands::battery_command::SetOutput`).
/// 3. **Constructs the correct command execution logic with its arguments**.
/// 4. **Processes the command using `process_command()`**.
/// 5. **Downcasts the result to the correct response type**.
#[proc_macro]
pub fn execute_command(input: TokenStream) -> TokenStream {
    let CommandInput {
        module,
        module_command_type,
        command_enum,
        command_variant,
        args,
    } = parse_macro_input!(input as CommandInput);
    let module_command_snake = quote!(#module_command_type)
        .to_string()
        .to_case(Case::Snake);
    let mod_module_commands = format_ident!("{}", module_command_snake);

    let command_creation = if let Some(args) = args {
        let args_named = args.iter().map(|kv| {
            let key = &kv.key;
            let value = &kv.value;
            quote! { #key: #value }
        });
        quote! { #command_enum { #(#args_named),* } }
    } else {
        // Handles unit variants
        quote! { #command_enum }
    };

    let expanded = quote! {
        {
            use amnio_firmware::modules::module::{Module, ModuleCommand, ModuleError};

            type CommandEnum = amnio_firmware::modules::commands::#mod_module_commands::#command_variant;
            type ResponseType = <CommandEnum as ModuleCommand>::Response;

            let command = #command_creation;

            let result: Result<Box<dyn std::any::Any>, ModuleError> = #module.process_command(command);
            let final_result: Result<ResponseType, ModuleError> = result.and_then(|boxed| {
                boxed
                    .downcast::<ResponseType>()
                    .map(|boxed_response| *boxed_response)
                    .map_err(|_| ModuleError::DowncastFailure)
            });

            final_result
        }
    };

    TokenStream::from(expanded)
}
