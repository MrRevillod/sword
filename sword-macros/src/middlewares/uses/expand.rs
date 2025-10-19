use proc_macro2::TokenStream;
use quote::quote;

use super::parse::MiddlewareArgs;

pub fn expand_middleware_args(args: &MiddlewareArgs) -> TokenStream {
    match args {
        MiddlewareArgs::SwordSimple(path) => {
            quote! {
                {
                    let middleware = state.borrow::<::std::sync::Arc<#path>>()
                        .expect("Failed to retrieve middleware from State");

                    ::sword::__internal::mw_with_state(
                        state.clone(),
                        move |ctx: ::sword::web::Request, next: ::sword::web::Next| {
                            let mw = ::std::sync::Arc::clone(&middleware);
                            async move {
                                mw.__on__request__function__(ctx, next).await
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
