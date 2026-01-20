mod generation;
mod on_handler;

use crate::shared::CommonHttpAdapterInput;
use generation::generate_socketio_adapter_builder;
use proc_macro::TokenStream;
use quote::quote;
use syn::{ItemStruct, Path};

pub use on_handler::expand_on_handler;

pub fn expand_socketio_adapter(
    attr: TokenStream,
    item: TokenStream,
) -> syn::Result<TokenStream> {
    let input = syn::parse::<ItemStruct>(item.clone())?;
    let parsed_input = CommonHttpAdapterInput::parse(attr, item)?;

    let interceptors: Vec<Path> = input
        .attrs
        .iter()
        .filter(|attr| attr.path().is_ident("interceptor"))
        .filter_map(|attr| attr.parse_args::<Path>().ok())
        .collect();

    let struct_name = parsed_input.struct_name.to_string();
    let namespace = &parsed_input.base_path;

    crate::shared::CMetaStack::push("socketio_adapter_name", &struct_name);
    crate::shared::CMetaStack::push("socketio_namespace", namespace);

    let interceptors_str = interceptors
        .iter()
        .map(|p| quote!(#p).to_string())
        .collect::<Vec<_>>()
        .join(";;;");

    crate::shared::CMetaStack::push("socketio_interceptors", &interceptors_str);

    let builder = generate_socketio_adapter_builder(&parsed_input, &interceptors);

    let expanded = quote! {
        #input
        #builder
    };

    Ok(TokenStream::from(expanded))
}
