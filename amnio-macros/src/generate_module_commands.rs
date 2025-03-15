use convert_case::{Case, Casing};
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{
    Ident, Result, Token, Type, parenthesized,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
};

// Represents a single command
struct Command {
    name: Ident,
    args: Vec<(Ident, Type)>,
    return_type: Option<Type>,
}

// Represents the entire macro input
pub struct ModuleCommandDef {
    enum_name: Ident,
    commands: Vec<Command>,
}

impl Parse for ModuleCommandDef {
    fn parse(input: ParseStream) -> Result<Self> {
        let enum_name: Ident = input.parse()?;
        let content;
        syn::braced!(content in input);

        let mut commands = Vec::new();
        while !content.is_empty() {
            let cmd_name: Ident = content.parse()?;

            let args = if content.peek(syn::token::Paren) {
                let args_content;
                parenthesized!(args_content in content);
                let args_punct: Punctuated<(Ident, Token![:], Type), Token![,]> = args_content
                    .parse_terminated(
                        |p| {
                            let name: Ident = p.parse()?;
                            let _: Token![:] = p.parse()?;
                            let ty: Type = p.parse()?;
                            Ok((name, Token![:](p.span()), ty))
                        },
                        Token![,],
                    )?; // Explicitly specify separator
                args_punct
                    .into_iter()
                    .map(|(name, _, ty)| (name, ty))
                    .collect()
            } else {
                Vec::new()
            };

            let return_type = if content.peek(Token![->]) {
                content.parse::<Token![->]>()?;
                Some(content.parse()?)
            } else {
                None
            };

            content.parse::<Token![;]>()?;

            commands.push(Command {
                name: cmd_name,
                args,
                return_type,
            });
        }

        Ok(ModuleCommandDef {
            enum_name,
            commands,
        })
    }
}

// Generates the Rust code for the macro
pub fn generate_module_commands(def: ModuleCommandDef) -> TokenStream {
    let enum_name = &def.enum_name;
    let module_name = format_ident!("{}", enum_name.to_string().to_case(Case::Snake));

    let enum_variants = def.commands.iter().map(|cmd| {
        let name = &cmd.name;
        if cmd.args.is_empty() {
            quote! { #name }
        } else {
            let args = cmd.args.iter().map(|(name, ty)| quote! { #name: #ty });
            quote! { #name { #(#args),* } }
        }
    });

    let struct_defs = def.commands.iter().map(|cmd| {
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

    let expanded = quote! {
        #[derive(Debug)]
        pub enum #enum_name {
            #(#enum_variants),*
        }

        pub mod #module_name {
            use crate::modules::module::ModuleCommand;

            #(#struct_defs)*
        }
    };

    TokenStream::from(expanded)
}
