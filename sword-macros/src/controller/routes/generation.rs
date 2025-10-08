use proc_macro2::TokenStream;
use quote::quote;
use syn::Type;

use crate::{
    controller::routes::{HTTP_METHODS, parsing::RouteInfo},
    middleware::expand_middleware_args,
};

pub fn generate_controller_routes(
    struct_self: &Type,
    routes: Vec<RouteInfo>,
) -> Result<TokenStream, syn::Error> {
    let mut handlers = vec![];

    for route in routes.iter() {
        let routing_function = match route.method.as_str() {
            "get" => quote! { axum_get_fn },
            "post" => quote! { axum_post_fn },
            "put" => quote! { axum_put_fn },
            "patch" => quote! { axum_patch_fn },
            "delete" => quote! { axum_delete_fn },
            _ => {
                return Err(syn::Error::new(
                    proc_macro2::Span::call_site(),
                    format!(
                        "Unsupported HTTP method. Only {} are supported",
                        HTTP_METHODS.join(", ")
                    ),
                ));
            }
        };

        let route_path = &route.path;
        let handler_name = &route.handler_name;

        let mut handler = if route.needs_context {
            quote! {
                ::sword::__internal::#routing_function({
                    let controller_build = std::sync::Arc::clone(&controller);

                    move |ctx: ::sword::web::Context| {
                        let controller_build = controller_build.clone();

                        async move {
                            use ::sword::__internal::IntoResponse;

                            match controller_build.as_ref() {
                                Ok(controller) => controller.#handler_name(ctx).await.into_response(),
                                Err(err) => ::sword::web::HttpResponse::InternalServerError()
                                    .message(format!("Controller build error: {err}"))
                                    .into_response(),
                            }
                        }
                    }
                })
            }
        } else {
            quote! {
                ::sword::__internal::#routing_function({
                    let controller_build = std::sync::Arc::clone(&controller);

                    move |_ctx: ::sword::web::Context| {
                        let controller_build = controller_build.clone();

                        async move {
                            use ::sword::__internal::IntoResponse;

                            match controller_build.as_ref() {
                                Ok(controller) => controller.#handler_name().await.into_response(),
                                Err(err) => ::sword::web::HttpResponse::InternalServerError()
                                    .message(format!("Controller build error: {err}"))
                                    .into_response(),
                            }
                        }
                    }
                })
            }
        };

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
        impl ::sword::web::Controller for #struct_self
        where
            Self: ::sword::web::ControllerBuilder
        {
            fn router(state: ::sword::core::State) -> ::sword::__internal::AxumRouter {
                let controller = std::sync::Arc::new(Self::build(state.clone()));

                let base_router = ::sword::__internal::AxumRouter::new()
                    #(#handlers)*
                    .with_state(state.clone());


                let base_path = #struct_self::base_path();
                let router = #struct_self::apply_controller_middlewares(base_router, state);

                match base_path {
                    "/" => router,
                    _ => ::sword::__internal::AxumRouter::new()
                        .nest(base_path, router),
                }
            }
        }
    })
}
