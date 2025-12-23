use proc_macro::TokenStream;
use syn::{Ident, ItemStruct, Type};

use crate::shared::StructFields;

pub struct MiddlewareInput {
    pub struct_name: Ident,
    pub fields: Vec<(Ident, Type)>,
}

pub fn parse_middleware_input(
    _: TokenStream,
    item: TokenStream,
) -> syn::Result<MiddlewareInput> {
    let input = syn::parse::<ItemStruct>(item)?;
    let fields = StructFields::parse(&input)?;

    Ok(MiddlewareInput {
        struct_name: input.ident,
        fields,
    })
}
