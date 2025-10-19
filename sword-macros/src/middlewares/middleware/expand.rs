use proc_macro::TokenStream;
use quote::quote;
use syn::ItemStruct;

use crate::middlewares::{generate_middleware_builder, parse_middleware_input};

pub fn expand_middleware(
    attr: TokenStream,
    item: TokenStream,
) -> Result<TokenStream, syn::Error> {
    let input = syn::parse::<ItemStruct>(item.clone())?;
    let parsed_input = parse_middleware_input(attr, item)?;
    let builder = generate_middleware_builder(&parsed_input);

    let expanded = quote! {
        #input
        #builder
    };

    Ok(TokenStream::from(expanded))
}
