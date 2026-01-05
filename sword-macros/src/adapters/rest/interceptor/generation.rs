use proc_macro2::TokenStream;
use quote::quote;

use super::InterceptorArgs;

/// Expands middleware arguments into the appropriate runtime code.
///
/// 1. **Simple middleware** (`#[uses(MyMiddleware)]`):
///    - Requires `MyMiddleware` to implement `OnRequest`
///
/// 2. **Middleware with config** (`#[uses(MyMiddleware(config))]`):
///    - Requires `MyMiddleware` to implement `OnRequestWithConfig<ConfigType>`
pub fn expand_interceptor_args(args: &InterceptorArgs) -> TokenStream {
    match args {
        InterceptorArgs::SwordSimple(path) => {
            quote! {
                {
                    fn __check_on_request<M: ::sword::prelude::OnRequest>(mw: &M) -> &M { mw }

                    let middleware = state.borrow::<::std::sync::Arc<#path>>()
                        .expect("Failed to retrieve middleware from State");

                    let _ = __check_on_request(&**middleware);

                    ::sword::internal::axum::mw_with_state(
                        state.clone(),
                        move |mut req: ::sword::prelude::Request, next: ::sword::prelude::Next| {
                            req.set_next(next);

                            let mw = ::std::sync::Arc::clone(&middleware);
                            async move {
                                <#path as ::sword::prelude::OnRequest>::on_request(&*mw, req).await
                            }
                        }
                    )
                }
            }
        }
        InterceptorArgs::SwordWithConfig { middleware, config } => {
            quote! {
                {
                    fn __check_on_request_with_config<M, C>(mw: &M) -> &M
                    where
                        M: ::sword::prelude::OnRequestWithConfig<C>
                    {
                        mw
                    }

                    let middleware = state.borrow::<::std::sync::Arc<#middleware>>()
                        .expect("Failed to retrieve middleware from State");

                    let _ = __check_on_request_with_config::<#middleware, _>(&**middleware);

                    ::sword::internal::axum::mw_with_state(
                        state.clone(),
                        move |mut req: ::sword::prelude::Request, next: ::sword::prelude::Next| {
                            req.set_next(next);
                            let mw = ::std::sync::Arc::clone(&middleware);
                            async move {
                                <#middleware as ::sword::prelude::OnRequestWithConfig<_>>::on_request_with_config(
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
        InterceptorArgs::Expression(expr) => {
            quote! { #expr }
        }
    }
}
