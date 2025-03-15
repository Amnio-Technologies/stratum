use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::token::{Brace, Colon};
use syn::{Expr, Ident, Path, Token, Type};

/// Represents a key-value pair inside `{}` for struct-like enum variants.
pub struct KeyValuePair {
    pub key: Ident,
    pub value: Expr,
}

impl Parse for KeyValuePair {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let key = input.parse()?;
        let _colon: Token![:] = input.parse()?;
        let value = input.parse()?;

        Ok(KeyValuePair { key, value })
    }
}

/// Parses input for the `execute_command!` macro
pub struct CommandInput {
    pub module: Expr,              // The module instance (e.g., dummy_module)
    pub module_command_type: Type, // The explicit module type (e.g., DummyModule)
    pub command_enum: Path,        // Full path (e.g., BatteryCommand::GetVoltage)
    pub command_variant: Ident,    // Extracted variant (e.g., GetVoltage)
    pub args: Option<Punctuated<KeyValuePair, Token![,]>>, //  Key-value arguments
}

impl Parse for CommandInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let module: Expr = input.parse()?; // Parse module
        input.parse::<Colon>()?; // Expect colon (`:`)

        let module_command_type: Type = input.parse()?; // Parse module type
        input.parse::<Token![,]>()?; // Expect comma

        let command_enum: Path = input.parse()?; // Parse full path (e.g., BatteryCommand::GetVoltage)

        // Extract the last segment as the command variant (e.g., GetVoltage)
        let command_variant = command_enum
            .segments
            .last()
            .ok_or_else(|| input.error("Expected a valid command variant"))?
            .ident
            .clone();

        let args = if input.peek(Brace) {
            let content;
            syn::braced!(content in input);

            // Parse key-value arguments
            let args_list = Punctuated::<KeyValuePair, Token![,]>::parse_terminated(&content)?;
            Some(args_list)
        } else {
            None
        };

        Ok(CommandInput {
            module,
            module_command_type,
            command_enum,
            command_variant,
            args,
        })
    }
}
