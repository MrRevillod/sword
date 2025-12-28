mod generation;
mod parsing;

use proc_macro::TokenStream;
use quote::quote;
use syn::ItemImpl;

use generation::generate_controller_routes;
use parsing::parse_routes;

pub fn expand_controller_routes(
    _: TokenStream,
    item: TokenStream,
) -> Result<TokenStream, syn::Error> {
    let item = syn::parse::<ItemImpl>(item)?;
    let parsed = parse_routes(&item)?;
    let generated = generate_controller_routes(&item.self_ty, &parsed)?;

    let expanded = quote! {
        #item
        #generated
    };

    Ok(TokenStream::from(expanded))
}
