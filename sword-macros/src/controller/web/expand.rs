use proc_macro::TokenStream as TokenStream1;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use crate::{
    common::{gen_build, gen_clone, gen_deps},
    controller::CommonControllerInput,
    controller::web::{ParsedRouteAttribute, RequestMode},
};

pub fn generate_route(parsed: ParsedRouteAttribute) -> TokenStream1 {
    let fn_name = parsed.function.sig.ident.clone();
    let controller_ident = format_ident!("{}", parsed.controller_name);
    let route_fn_name = format_ident!("__sword_route_{}", fn_name);

    let handler = generate_handler(&parsed);
    let handler_with_interceptors =
        apply_interceptors(handler, parsed.request_mode, &parsed);
    let inventory_registration =
        generate_inventory_registration(&parsed, &controller_ident, &route_fn_name);

    let input_fn = &parsed.function;
    let expanded = quote! {
        #input_fn

        pub fn #route_fn_name(
            controller: std::sync::Arc<Self>,
            state: ::sword::internal::core::State,
        ) -> ::sword::internal::axum::MethodRouter<::sword::internal::core::State> {
            #handler_with_interceptors
        }

        #inventory_registration
    };

    TokenStream1::from(expanded)
}

pub fn generate_controller_builder(input: &CommonControllerInput) -> TokenStream {
    let base_path = &input.base_path;
    let self_name = &input.struct_name;
    let self_fields = &input.fields;
    let controller_interceptors = &input.interceptors;

    let processed_interceptors: Vec<TokenStream> = controller_interceptors
        .iter()
        .map(|interceptor| {
            expand_interceptor_args(interceptor, RequestMode::Buffered)
        })
        .collect();

    let deps_impl = gen_deps(self_name, self_fields);
    let build_impl = gen_build(self_name, self_fields);
    let clone_impl = gen_clone(self_name, self_fields);

    quote! {
        #build_impl
        #deps_impl
        #clone_impl

        impl ::sword::internal::core::WebController for #self_name {
            fn base_path() -> &'static str {
                #base_path
            }

            fn apply_interceptors(
                router: ::sword::internal::axum::AxumRouter<::sword::internal::core::State>,
                state: ::sword::internal::core::State,
            ) -> ::sword::internal::axum::AxumRouter<::sword::internal::core::State> {
                let mut result = router;
                #(
                    result = result.layer(#processed_interceptors);
                )*
                result
            }
        }

        impl ::sword::internal::core::ControllerSpec for #self_name {
            fn kind() -> ::sword::internal::core::Controller {
                ::sword::internal::core::Controller::Web
            }

            fn type_id() -> ::std::any::TypeId {
                ::std::any::TypeId::of::<#self_name>()
            }
        }
    }
}

pub fn expand_interceptor_args(
    args: &crate::interceptor::InterceptorArgs,
    request_mode: RequestMode,
) -> TokenStream {
    let mode = mode_tokens(request_mode);

    match args {
        crate::interceptor::InterceptorArgs::SwordSimple(path) => {
            generate_sword_simple(path, &mode)
        }
        crate::interceptor::InterceptorArgs::SwordWithConfig {
            middleware,
            config,
        } => generate_sword_configured(middleware, config, &mode),
        crate::interceptor::InterceptorArgs::Expression(expr) => {
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
                        hints: ["Ensure the interceptor is registered and built before controller initialization"],
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
                        hints: ["Ensure the interceptor is registered and built before controller initialization"],
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

fn generate_handler(parsed: &ParsedRouteAttribute) -> TokenStream {
    let routing_fn = match parsed.method.as_str() {
        "GET" => quote! { get },
        "POST" => quote! { post },
        "PUT" => quote! { put },
        "DELETE" => quote! { delete },
        "PATCH" => quote! { patch },
        _ => quote! { get },
    };

    let fn_name = &parsed.function.sig.ident;
    let params: Vec<_> = parsed
        .function
        .sig
        .inputs
        .iter()
        .filter_map(|arg| {
            if let syn::FnArg::Typed(pat_type) = arg {
                Some((pat_type.pat.clone(), pat_type.ty.clone()))
            } else {
                None
            }
        })
        .collect();

    if params.is_empty() {
        quote! {
            ::sword::internal::axum::routing::#routing_fn({
                let ctrl = std::sync::Arc::clone(&controller);
                move || async move {
                    use ::sword::internal::axum::IntoResponse;
                    ctrl.#fn_name().await.into_response()
                }
            })
        }
    } else {
        let closure_params: Vec<_> = params
            .iter()
            .enumerate()
            .map(|(i, (_, ty))| {
                let param_name = format_ident!("p{}", i);
                quote! { #param_name: #ty }
            })
            .collect();

        let call_args: Vec<_> = (0..params.len())
            .map(|i| {
                let param_name = format_ident!("p{}", i);
                quote! { #param_name }
            })
            .collect();

        quote! {
            ::sword::internal::axum::routing::#routing_fn({
                let ctrl = std::sync::Arc::clone(&controller);
                move |#(#closure_params),*| async move {
                    use ::sword::internal::axum::IntoResponse;
                    ctrl.#fn_name(#(#call_args),*).await.into_response()
                }
            })
        }
    }
}

fn apply_interceptors(
    mut handler: TokenStream,
    request_mode: RequestMode,
    parsed: &ParsedRouteAttribute,
) -> TokenStream {
    for interceptor in parsed.interceptors.iter().rev() {
        let generated_interceptor =
            expand_interceptor_args(interceptor, request_mode);
        handler = quote! {
            #handler.layer(#generated_interceptor)
        };
    }
    handler
}

fn generate_inventory_registration(
    parsed: &ParsedRouteAttribute,
    controller_ident: &syn::Ident,
    route_fn_name: &syn::Ident,
) -> TokenStream {
    let fn_name = &parsed.function.sig.ident;
    let registration_name = format_ident!(
        "__SWORD_ROUTE_REGISTRAR_{}_{}",
        parsed.controller_name.replace("::", "_"),
        fn_name
    );

    let controller_name = &parsed.controller_name;
    let controller_path = &parsed.controller_path;
    let route_path = &parsed.path;
    let method = &parsed.method;

    quote! {
        #[allow(non_upper_case_globals)]
        #[doc(hidden)]
        const #registration_name: () = {
            ::sword::internal::inventory::submit! {
                ::sword::internal::controllers::RouteRegistrar {
                    controller_id: ::std::any::TypeId::of::<#controller_ident>(),
                    controller_path: #controller_path,
                    path: #route_path,
                    handler: |state: ::sword::internal::core::State| -> ::sword::internal::axum::MethodRouter<::sword::internal::core::State> {
                        let controller = std::sync::Arc::new(
                            #controller_ident::build(&state).unwrap_or_else(|err| {
                                ::sword::internal::core::sword_error!(
                                    title: "Failed to build HTTP controller",
                                    reason: err,
                                    context: {
                                        "controller" => #controller_name,
                                        "route" => format!("{} {}", #method, #route_path),
                                    },
                                    hints: ["Ensure all controller dependencies are registered in the DI container"],
                                )
                            })
                        );

                        #controller_ident::#route_fn_name(controller, state)
                    },
                    apply_top_level_interceptors: #controller_ident::apply_interceptors,
                }
            }
        };
    }
}
