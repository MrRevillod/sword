use proc_macro2::TokenStream;
use quote::quote;

use super::parse::MiddlewareArgs;

/// Expands middleware arguments into the appropriate runtime code.
///
/// 1. **Simple middleware** (`#[uses(MyMiddleware)]`):
///    - Requires `MyMiddleware` to implement `OnRequest`
///
/// 2. **Middleware with config** (`#[uses(MyMiddleware(config))]`):
///    - Requires `MyMiddleware` to implement `OnRequestWithConfig<ConfigType>`
pub fn expand_middleware_args(args: &MiddlewareArgs) -> TokenStream {
    match args {
        MiddlewareArgs::SwordSimple(path) => {
            quote! {
                {
                    fn __check_on_request<M: ::sword::web::OnRequest>(mw: &M) -> &M { mw }

                    let middleware = state.borrow::<::std::sync::Arc<#path>>()
                        .expect("Failed to retrieve middleware from State");

                    let _ = __check_on_request(&**middleware);

                    ::sword::internal::mw_with_state(
                        state.clone(),
                        move |mut req: ::sword::web::Request, next: ::sword::web::Next| {
                            req.set_next(next);

                            let mw = ::std::sync::Arc::clone(&middleware);
                            async move {
                                <#path as ::sword::web::OnRequest>::on_request(&*mw, req).await
                            }
                        }
                    )
                }
            }
        }
        MiddlewareArgs::SwordWithConfig { middleware, config } => {
            quote! {
                {
                    fn __check_on_request_with_config<M, C>(mw: &M) -> &M
                    where
                        M: ::sword::web::OnRequestWithConfig<C>
                    {
                        mw
                    }

                    let middleware = state.borrow::<::std::sync::Arc<#middleware>>()
                        .expect("Failed to retrieve middleware from State");

                    let _ = __check_on_request_with_config::<#middleware, _>(&**middleware);

                    ::sword::internal::mw_with_state(
                        state.clone(),
                        move |mut req: ::sword::web::Request, next: ::sword::web::Next| {
                            req.set_next(next);
                            let mw = ::std::sync::Arc::clone(&middleware);
                            async move {
                                <#middleware as ::sword::web::OnRequestWithConfig<_>>::on_request_with_config(
                                    &*mw,
                                    #config,
                                    req,
                                ).await
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
