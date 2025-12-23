use crate::{
    shared::CommonHttpAdapterInput,
    socketio::generation::generate_socketio_adapter_builder,
};

use proc_macro::TokenStream;
use quote::quote;
use syn::ItemStruct;

pub fn expand_socketio_adapter(
    attr: TokenStream,
    item: TokenStream,
) -> syn::Result<TokenStream> {
    let input = syn::parse::<ItemStruct>(item.clone())?;
    let parsed_input = CommonHttpAdapterInput::parse(attr, item)?;

    let builder = generate_socketio_adapter_builder(&parsed_input);

    let expanded = quote! {
        #input
        #builder
    };

    Ok(TokenStream::from(expanded))
}
