use proc_macro2::TokenStream;
use quote::quote;

use super::InterceptorArgs;
use crate::adapters::controllers::attributes::RequestMode;

/// Expands interceptor arguments into the appropriate runtime code.
///
/// 1. **Simple interceptor** (`#[interceptor(MyInterceptor)]`):
///    - Requires `MyInterceptor` to implement `OnRequest` or `OnRequestStream`
///
/// 2. **Configured interceptor** (`#[interceptor(MyInterceptor, config = ...)]`):
///    - Requires `MyInterceptor` to implement
///      `OnRequestWithConfig<ConfigType>` or `OnRequestStreamWithConfig<ConfigType>`
pub fn expand_interceptor_args(
    args: &InterceptorArgs,
    request_mode: RequestMode,
) -> TokenStream {
    let mode = mode_tokens(request_mode);

    match args {
        InterceptorArgs::SwordSimple(path) => generate_sword_simple(path, &mode),
        InterceptorArgs::SwordWithConfig { middleware, config } => {
            generate_sword_configured(middleware, config, &mode)
        }
        InterceptorArgs::Expression(expr) => {
            quote! { #expr }
        }
    }
}

struct ModeTokens {
    req_ty: TokenStream,
    simple_trait: TokenStream,
    configured_trait: TokenStream,
}

fn mode_tokens(request_mode: RequestMode) -> ModeTokens {
    match request_mode {
        RequestMode::Streaming => ModeTokens {
            req_ty: quote! { ::sword::prelude::StreamRequest },
            simple_trait: quote! { ::sword::prelude::OnRequestStream },
            configured_trait: quote! { ::sword::prelude::OnRequestStreamWithConfig },
        },
        _ => ModeTokens {
            req_ty: quote! { ::sword::prelude::Request },
            simple_trait: quote! { ::sword::prelude::OnRequest },
            configured_trait: quote! { ::sword::prelude::OnRequestWithConfig },
        },
    }
}

fn generate_sword_simple(path: &syn::Path, mode: &ModeTokens) -> TokenStream {
    let req_ty = &mode.req_ty;
    let simple_trait = &mode.simple_trait;

    quote! {
        {
            fn __check_on_request<M: #simple_trait>(mw: &M) -> &M { mw }

            let middleware = state.borrow::<#path>()
                .unwrap_or_else(|err| {
                    ::sword::internal::core::sword_error!(
                        title: "Failed to retrieve HTTP interceptor from State",
                        reason: err,
                        context: {
                            "interceptor" => stringify!(#path),
                        },
                        hints: ["Ensure the interceptor is registered and built before adapter initialization"],
                    )
                });

            let _ = __check_on_request(&*middleware);

            ::sword::internal::axum::mw_with_state(
                state.clone(),
                move |mut req: #req_ty, next: ::sword::prelude::Next| {
                    req.set_next(next);

                    let mw = ::std::sync::Arc::clone(&middleware);
                    async move {
                        <#path as #simple_trait>::on_request(&*mw, req).await
                    }
                }
            )
        }
    }
}

fn generate_sword_configured(
    middleware_ty: &syn::Path,
    config: &syn::Expr,
    mode: &ModeTokens,
) -> TokenStream {
    let req_ty = &mode.req_ty;
    let configured_trait = &mode.configured_trait;

    quote! {
        {
            fn __check_on_request_configured<M, C>(mw: &M) -> &M
            where
                M: #configured_trait<C>
            {
                mw
            }

            let middleware = state.borrow::<#middleware_ty>()
                .unwrap_or_else(|err| {
                    ::sword::internal::core::sword_error!(
                        title: "Failed to retrieve HTTP interceptor from State",
                        reason: err,
                        context: {
                            "interceptor" => stringify!(#middleware_ty),
                        },
                        hints: ["Ensure the interceptor is registered and built before adapter initialization"],
                    )
                });

            let _ = __check_on_request_configured::<#middleware_ty, _>(&*middleware);

            ::sword::internal::axum::mw_with_state(
                state.clone(),
                move |mut req: #req_ty, next: ::sword::prelude::Next| {
                    req.set_next(next);
                    let mw = ::std::sync::Arc::clone(&middleware);
                    async move {
                        <#middleware_ty as #configured_trait<_>>::on_request(
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
