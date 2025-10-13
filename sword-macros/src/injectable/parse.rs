use crate::shared::collect_struct_fields;
use proc_macro::TokenStream;
use syn::parse::{ParseStream, Result as ParseResult};
use syn::{Ident, ItemStruct, LitStr, Token, Type, parse::Parse};

pub enum InjectableKind {
    Provider,
    Component,
}

pub struct InjectableStructInput {
    pub struct_name: Ident,
    pub fields: Vec<(Ident, Type)>,
    pub derive_clone: bool,
    pub kind: InjectableKind,
}

struct InjectableArgs {
    kind: InjectableKind,
    derive_clone: bool,
}

impl Parse for InjectableArgs {
    fn parse(input: ParseStream) -> ParseResult<Self> {
        let mut kind = InjectableKind::Component;
        let mut derive_clone = true;

        if input.is_empty() {
            return Ok(Self { kind, derive_clone });
        }

        while !input.is_empty() {
            let arg: Ident = input.parse()?;

            match arg.to_string().as_str() {
                "kind" => {
                    input.parse::<Token![=]>()?;
                    let val: LitStr = input.parse()?;
                    kind = match val.value().as_str() {
                        "provider" => InjectableKind::Provider,
                        "component" => InjectableKind::Component,
                        _ => {
                            return Err(syn::Error::new_spanned(
                                val,
                                "Expected 'provider' or 'component'",
                            ));
                        }
                    };
                }
                "no_derive_clone" => derive_clone = false,
                _ => return Err(syn::Error::new_spanned(arg, "Unknown attribute")),
            }

            if !input.is_empty() {
                input.parse::<Token![,]>()?;
            }
        }

        Ok(Self { kind, derive_clone })
    }
}

pub fn parse_dependency_struct_input(
    attr: TokenStream,
    item: TokenStream,
) -> Result<InjectableStructInput, syn::Error> {
    let input = syn::parse::<ItemStruct>(item)?;
    let args = syn::parse::<InjectableArgs>(attr)?;

    Ok(InjectableStructInput {
        struct_name: input.clone().ident,
        fields: collect_struct_fields(&input),
        derive_clone: args.derive_clone,
        kind: args.kind,
    })
}
