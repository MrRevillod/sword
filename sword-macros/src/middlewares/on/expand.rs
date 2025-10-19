use proc_macro::TokenStream;
use quote::quote;
use syn::ItemFn;

use crate::middlewares::parse_on_middleware_function;

pub fn expand_on_request(
    _: TokenStream,
    item: TokenStream,
) -> Result<TokenStream, syn::Error> {
    let input = syn::parse::<ItemFn>(item.clone())?;
    let parsed = parse_on_middleware_function(&input)?;

    let args = &parsed.fn_arguments;
    let body = &parsed.fn_body;
    let output = &parsed.fn_output;

    let generated = quote! {
        #[allow(non_snake_case)]
        pub async fn __on__request__function__(#(#args),*) #output {
            #body
        }
    };

    let expanded = quote! {
        #generated
    };

    Ok(TokenStream::from(expanded))
}
