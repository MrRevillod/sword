mod generation;

use proc_macro::TokenStream;
use quote::quote;
use syn::ItemStruct;

use crate::shared::{CMetaStack, CommonHttpAdapterInput};
use generation::generate_controller_builder;

pub fn expand_controller(
    attr: TokenStream,
    item: TokenStream,
) -> syn::Result<TokenStream> {
    let input = syn::parse::<ItemStruct>(item.clone())?;
    let parsed_input = CommonHttpAdapterInput::parse(attr, item)?;

    let struct_name = parsed_input.struct_name.to_string();
    let base_path = &parsed_input.base_path;

    CMetaStack::push("controller_name", &struct_name);
    CMetaStack::push("controller_path", base_path);

    let builder = generate_controller_builder(&parsed_input);

    let expanded = quote! {
        #input
        #builder
    };

    Ok(TokenStream::from(expanded))
}
