use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::Type;

use crate::{
    controller::routes::{HTTP_METHODS, parsing::RouteInfo},
    middlewares::expand_middleware_args,
};

pub fn generate_controller_routes(
    struct_ty: &Type,
    routes: &[RouteInfo],
) -> Result<TokenStream, syn::Error> {
    let mut handlers = vec![];

    for route in routes {
        let route_path = &route.path;
        let mut handler = generate_handler(route)?;

        for middleware in route.middlewares.iter().rev() {
            let generated_middleware = expand_middleware_args(middleware);

            handler = quote! {
                #handler.layer(#generated_middleware)
            };
        }

        handlers.push(quote! {
            .route(#route_path, #handler)
        });
    }

    Ok(quote! {

        impl #struct_ty {
            fn router_builder(state: ::sword::core::State) -> ::sword::internal::AxumRouter {
                let controller = std::sync::Arc::new(
                    Self::build(&state).unwrap_or_else(|err| {
                        panic!("\n❌ Failed to build controller\n\n{}\n", err)
                    })
                );

                let base_router = ::sword::internal::AxumRouter::new()
                    #(#handlers)*
                    .with_state(state.clone());

                let base_path = #struct_ty::base_path();
                let router = #struct_ty::apply_middlewares(base_router, state);

                match base_path {
                    "/" => router,
                    _ => ::sword::internal::AxumRouter::new().nest(base_path, router),
                }

            }
        }

        impl ::sword::core::Gateway for #struct_ty
        where
            Self: ::sword::web::Controller
        {
            fn kind() -> ::sword::core::GatewayKind {
                ::sword::core::GatewayKind::Rest(Box::new(Self::router_builder))
            }
        }
    })
}

fn generate_axum_routing_fn(method: &str) -> syn::Result<TokenStream> {
    let routing_fn = match method {
        "get" => quote! { axum_get_fn },
        "post" => quote! { axum_post_fn },
        "put" => quote! { axum_put_fn },
        "patch" => quote! { axum_patch_fn },
        "delete" => quote! { axum_delete_fn },
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

    let handler = if route.needs_context {
        quote! {
            ::sword::internal::#routing_function({
                let ctrl = std::sync::Arc::clone(&controller);

                move |req: ::sword::web::Request| {
                    async move {
                        use ::sword::internal::IntoResponse;
                        ctrl.#handler_name(req).await.into_response()
                    }
                }
            })
        }
    } else {
        quote! {
            ::sword::internal::#routing_function({
                let ctrl = std::sync::Arc::clone(&controller);

                move |_: ::sword::web::Request| {
                    async move {
                        use ::sword::internal::IntoResponse;
                        ctrl.#handler_name().await.into_response()
                    }
                }
            })
        }
    };

    Ok(handler)
}
