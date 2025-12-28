mod generation;

use proc_macro::TokenStream;
use quote::quote;
use syn::ItemStruct;

use crate::shared::CommonHttpAdapterInput;
use generation::generate_controller_builder;

pub fn expand_controller(
    attr: TokenStream,
    item: TokenStream,
) -> syn::Result<TokenStream> {
    let input = syn::parse::<ItemStruct>(item.clone())?;
    let parsed_input = CommonHttpAdapterInput::parse(attr, item)?;
    let builder = generate_controller_builder(&parsed_input);

    let expanded = quote! {
        #input
        #builder
    };

    Ok(TokenStream::from(expanded))
}
