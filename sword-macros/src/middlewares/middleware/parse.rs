use proc_macro::TokenStream;
use syn::{Ident, ItemStruct, Type};

use crate::shared::collect_struct_fields;

pub struct MiddlewareInput {
    pub struct_name: Ident,
    pub fields: Vec<(Ident, Type)>,
}

pub fn parse_middleware_input(
    _: TokenStream,
    item: TokenStream,
) -> Result<MiddlewareInput, syn::Error> {
    let input = syn::parse::<ItemStruct>(item)?;
    let fields = collect_struct_fields(&input);

    Ok(MiddlewareInput {
        struct_name: input.ident,
        fields,
    })
}
