use proc_macro2::TokenStream;
use quote::quote;

use super::parse::MiddlewareArgs;

pub fn expand_middleware_args(args: &MiddlewareArgs) -> TokenStream {
    match args {
        MiddlewareArgs::SwordSimple(path) => {
            quote! {
                ::sword::__internal::mw_with_state(
                    state.clone(),
                    |ctx: ::sword::web::Context, next: ::sword::web::Next| async move {
                        <#path>::handle(ctx, next).await
                    }
                )
            }
        }
        MiddlewareArgs::SwordWithConfig { middleware, config } => {
            quote! {
                ::sword::__internal::mw_with_state(
                    state.clone(),
                    |mut ctx: ::sword::web::Context, next: ::sword::web::Next| async move {
                        <#middleware>::handle(#config, ctx, next).await
                    }
                )
            }
        }
        MiddlewareArgs::Expression(expr) => {
            quote! { #expr }
        }
    }
}
