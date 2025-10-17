use proc_macro2::TokenStream;
use quote::quote;

use super::parse::MiddlewareArgs;

pub fn expand_middleware_args(args: &MiddlewareArgs) -> TokenStream {
    match args {
        MiddlewareArgs::SwordSimple(path) => {
            quote! {
                {
                    let middleware = state.get::<#path>()
                        .expect("Failed to retrieve middleware from State");

                    ::sword::__internal::mw_with_state(
                        state.clone(),
                        move |ctx: ::sword::web::Context, next: ::sword::web::Next| {
                            let mw = middleware.clone();
                            async move {
                                mw.on_request(ctx, next).await
                            }
                        }
                    )
                }
            }
        }
        MiddlewareArgs::Expression(expr) => {
            quote! { #expr }
        }
    }
}
