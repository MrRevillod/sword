use proc_macro::TokenStream;
use quote::quote;
use syn::ItemStruct;

use crate::di::{injectable::generate_injectable_trait, *};

pub fn expand_injectable(
    attr: TokenStream,
    item: TokenStream,
) -> Result<TokenStream, syn::Error> {
    let input = syn::parse::<ItemStruct>(item.clone())?;
    let parsed = parse_dependency_struct_input(attr, item)?;

    let injectable_impl = generate_injectable_trait(&parsed);
    let clone_impl = generate_clone_impl(&parsed);

    let mut expanded = quote! {
        #input
        #injectable_impl
    };

    if parsed.derive_clone {
        expanded.extend(quote! {
            #clone_impl
        });
    }

    Ok(expanded.into())
}
