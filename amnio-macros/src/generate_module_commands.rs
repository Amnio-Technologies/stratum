use convert_case::{Case, Casing};
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{
    Ident, Result, Token, Type, parenthesized,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
};

// Represents a single command within a module
struct Command {
    name: Ident,
    args: Vec<(Ident, Type)>,
    return_type: Option<Type>,
}

// Represents a single module definition inside the macro
struct ModuleCommandDef {
    enum_name: Ident,
    commands: Vec<Command>,
}

// Represents multiple module definitions within the macro
pub struct ModuleCommandsDefList {
    modules: Punctuated<ModuleCommandDef, Token![,]>,
}

impl Parse for Command {
    fn parse(input: ParseStream) -> Result<Self> {
        let cmd_name: Ident = input.parse()?;

        let args = if input.peek(syn::token::Paren) {
            let args_content;
            parenthesized!(args_content in input);
            let args_punct: Punctuated<(Ident, Token![:], Type), Token![,]> = args_content
                .parse_terminated(
                    |p| {
                        let name: Ident = p.parse()?;
                        let _: Token![:] = p.parse()?;
                        let ty: Type = p.parse()?;
                        Ok((name, Token![:](p.span()), ty))
                    },
                    Token![,],
                )?;
            args_punct
                .into_iter()
                .map(|(name, _, ty)| (name, ty))
                .collect()
        } else {
            Vec::new()
        };

        let return_type = if input.peek(Token![->]) {
            input.parse::<Token![->]>()?;
            Some(input.parse()?)
        } else {
            None
        };

        input.parse::<Token![;]>()?;

        Ok(Command {
            name: cmd_name,
            args,
            return_type,
        })
    }
}

impl Parse for ModuleCommandDef {
    fn parse(input: ParseStream) -> Result<Self> {
        let enum_name: Ident = input.parse()?;
        let content;
        syn::braced!(content in input);

        let mut commands = Vec::new();
        while !content.is_empty() {
            commands.push(content.parse()?);
        }

        Ok(ModuleCommandDef {
            enum_name,
            commands,
        })
    }
}

impl Parse for ModuleCommandsDefList {
    fn parse(input: ParseStream) -> Result<Self> {
        let modules = Punctuated::parse_terminated(input)?;
        Ok(ModuleCommandsDefList { modules })
    }
}

// Generates the Rust code for the macro
pub fn generate_module_commands(defs: ModuleCommandsDefList) -> TokenStream {
    let module_code = defs.modules.iter().map(|module| {
        let enum_name = &module.enum_name;
        let module_name = format_ident!("{}", enum_name.to_string().to_case(Case::Snake));

        let enum_variants = module.commands.iter().map(|cmd| {
            let name = &cmd.name;
            if cmd.args.is_empty() {
                quote! { #name }
            } else {
                let args = cmd.args.iter().map(|(name, ty)| quote! { #name: #ty });
                quote! { #name { #(#args),* } }
            }
        });

        let struct_defs = module.commands.iter().map(|cmd| {
            let name = &cmd.name;
            let response_type = cmd
                .return_type
                .as_ref()
                .map_or(quote! { () }, |ty| quote! { #ty });

            quote! {
                pub struct #name;

                impl crate::modules::module::ModuleCommand for #name {
                    type Response = #response_type;

                    fn as_any(&self) -> &dyn std::any::Any {
                        self
                    }
                }
            }
        });

        quote! {
            #[derive(Debug)]
            pub enum #enum_name {
                #(#enum_variants),*
            }

            pub mod #module_name {
                use crate::modules::module::ModuleCommand;

                #(#struct_defs)*
            }
        }
    });

    let expanded = quote! {
        #(#module_code)*
    };

    TokenStream::from(expanded)
}
