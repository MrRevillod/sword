use super::parsing::ParsedRouteAttribute;
use crate::adapters::expand_interceptor_args;

use proc_macro::TokenStream as TokenStream1;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

pub fn generate_route(parsed: ParsedRouteAttribute) -> TokenStream1 {
    let fn_name = parsed.function.sig.ident.clone();
    let controller_ident = format_ident!("{}", parsed.controller_name);
    let route_fn_name = format_ident!("__sword_route_{}", fn_name);

    let handler = generate_handler(&parsed);
    let handler_with_interceptors = apply_interceptors(handler, &parsed);
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
    parsed: &ParsedRouteAttribute,
) -> TokenStream {
    for interceptor in parsed.interceptors.iter().rev() {
        let generated_interceptor = expand_interceptor_args(interceptor);
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

    quote! {
        #[allow(non_upper_case_globals)]
        #[doc(hidden)]
        const #registration_name: () = {
            ::sword::internal::inventory::submit! {
                ::sword::internal::rest::RouteRegistrar {
                    controller_type_id: ::std::any::TypeId::of::<#controller_ident>(),
                    controller_path: #controller_path,
                    route_path: #route_path,
                    handler: |state: ::sword::internal::core::State| -> ::sword::internal::axum::MethodRouter<::sword::internal::core::State> {
                        let controller = std::sync::Arc::new(
                            #controller_ident::build(&state).unwrap_or_else(|err| {
                                panic!("\n[!] Failed to build controller '{}'\n\n{}\n", #controller_name, err)
                            })
                        );

                        #controller_ident::#route_fn_name(controller, state)
                    },
                    apply_controller_level_interceptors: #controller_ident::apply_interceptors,
                }
            }
        };
    }
}
