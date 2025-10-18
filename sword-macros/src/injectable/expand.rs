use proc_macro::TokenStream;
use quote::quote;
use syn::ItemStruct;

use crate::{injectable::*, shared::generate_clone_impl};

pub fn expand_injectable(
    attr: TokenStream,
    item: TokenStream,
) -> Result<TokenStream, syn::Error> {
    let input = syn::parse::<ItemStruct>(item.clone())?;
    let parsed = parse_injectable_input(attr, item)?;

    let injectable_impl = match parsed.kind {
        InjectableKind::Provider => generate_provider_trait(&parsed),
        InjectableKind::Component => generate_component_trait(&parsed),
    };

    let clone_impl = generate_clone_impl(&parsed.struct_name, &parsed.fields);

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
