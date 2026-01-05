use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::Type;

use crate::adapters::rest::{
    interceptor::expand_interceptor_args,
    routes::parsing::{HTTP_METHODS, RouteInfo},
};

pub fn generate_controller_routes(
    struct_ty: &Type,
    routes: &[RouteInfo],
) -> syn::Result<TokenStream> {
    let mut handlers = vec![];

    for route in routes {
        let route_path = &route.path;
        let mut handler = generate_handler(route)?;

        for interceptor in route.interceptors.iter().rev() {
            let generated_interceptor = expand_interceptor_args(interceptor);

            handler = quote! {
                #handler.layer(#generated_interceptor)
            };
        }

        handlers.push(quote! {
            .route(#route_path, #handler)
        });
    }

    Ok(quote! {
        impl #struct_ty {
            fn router_builder(state: ::sword::internal::core::State) -> ::sword::internal::axum::AxumRouter {
                let controller = std::sync::Arc::new(
                    Self::build(&state).unwrap_or_else(|err| {
                        panic!("\n[!] Failed to build controller\n\n{}\n", err)
                    })
                );

                let base_router = ::sword::internal::axum::AxumRouter::new()
                    #(#handlers)*
                    .with_state(state.clone());

                let base_path = #struct_ty::base_path();
                let router = #struct_ty::apply_interceptors(base_router, state);

                match base_path {
                    "/" => router,
                    _ => ::sword::internal::axum::AxumRouter::new().nest(base_path, router),
                }

            }
        }

        impl ::sword::internal::core::Adapter for #struct_ty {
            fn kind() -> ::sword::internal::core::AdapterKind {
                ::sword::internal::core::AdapterKind::Rest(Box::new(Self::router_builder))
            }
        }
    })
}

fn generate_axum_routing_fn(method: &str) -> syn::Result<TokenStream> {
    let routing_fn = match method {
        "get" => quote! { get_fn },
        "post" => quote! { post_fn },
        "put" => quote! { put_fn },
        "patch" => quote! { patch_fn },
        "delete" => quote! { delete_fn },
        _ => {
            return Err(syn::Error::new(
                Span::call_site(),
                format!(
                    "Unsupported HTTP method. Only {} are supported",
                    HTTP_METHODS.join(", ")
                ),
            ));
        }
    };

    Ok(routing_fn)
}

fn generate_handler(route: &RouteInfo) -> syn::Result<TokenStream> {
    let routing_function = generate_axum_routing_fn(&route.method)?;

    let RouteInfo { handler_name, .. } = route;

    let req_param = route
        .needs_context
        .then(|| quote! { req: ::sword::prelude::Request })
        .into_iter();

    let req_arg = route.needs_context.then(|| quote! { req }).into_iter();

    let handler = quote! {
        ::sword::internal::axum::#routing_function({
            let ctrl = std::sync::Arc::clone(&controller);
            move |#(#req_param)*| {
                async move {
                    use ::sword::internal::axum::IntoResponse;
                    ctrl.#handler_name(#(#req_arg)*).await.into_response()
                }
            }
        })
    };

    Ok(handler)
}
