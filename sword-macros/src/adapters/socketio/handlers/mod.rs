mod generation;
mod parsing;

use generation::generate_socketio_handlers;
use parsing::parse_handlers;

use proc_macro::TokenStream;
use quote::quote;
use syn::ItemImpl;

pub fn expand_socketio_handlers(
    _: TokenStream,
    item: TokenStream,
) -> syn::Result<TokenStream> {
    let input = syn::parse::<ItemImpl>(item)?;
    let handlers = parse_handlers(&input)?;

    let struct_ty = &input.self_ty;
    let generated = generate_socketio_handlers(struct_ty, &handlers)?;

    let expanded = quote! {
        #input
        #generated
    };

    Ok(TokenStream::from(expanded))
}
