mod generation;

use proc_macro::TokenStream;
use quote::quote;
use syn::ItemStruct;

#[cfg(feature = "socketio-controllers")]
use crate::controllers::socketio::generate_socketio_controller_builder;
use crate::shared::{CMetaStack, CommonControllerInput, ParsedControllerKind};
use generation::generate_controller_builder;

fn has_sword_interceptors(
    interceptors: &[crate::controllers::web::InterceptorArgs],
) -> bool {
    interceptors.iter().any(|interceptor| {
        matches!(
            interceptor,
            crate::controllers::web::InterceptorArgs::SwordSimple(_)
                | crate::controllers::web::InterceptorArgs::SwordWithConfig { .. }
        )
    })
}

pub fn expand_controller(
    attr: TokenStream,
    item: TokenStream,
) -> syn::Result<TokenStream> {
    let input = syn::parse::<ItemStruct>(item.clone())?;
    let parsed_input = CommonControllerInput::parse(attr, item)?;

    let struct_name = parsed_input.struct_name.to_string();
    let mount_path = &parsed_input.base_path;

    let builder = match parsed_input.kind {
        ParsedControllerKind::Web => {
            let controller_has_sword_interceptors =
                has_sword_interceptors(&parsed_input.interceptors).to_string();

            CMetaStack::push("controller_name", &struct_name);
            CMetaStack::push("controller_path", mount_path);
            CMetaStack::push("controller_kind", "web");
            CMetaStack::push(
                "controller_has_sword_interceptors",
                &controller_has_sword_interceptors,
            );

            generate_controller_builder(&parsed_input)
        }
        ParsedControllerKind::SocketIo => {
            #[cfg(not(feature = "socketio-controllers"))]
            {
                return Err(syn::Error::new(
                    input.ident.span(),
                    "Controller::SocketIo requires enabling the `socketio-controllers` feature",
                ));
            }

            #[cfg(feature = "socketio-controllers")]
            {
                CMetaStack::push("controller_name", &struct_name);
                CMetaStack::push("controller_path", mount_path);
                CMetaStack::push("controller_kind", "socketio");

                // Keys consumed by #[on]
                CMetaStack::push("socketio_controller_name", &struct_name);
                CMetaStack::push("socketio_namespace", mount_path);

                generate_socketio_controller_builder(
                    &parsed_input,
                    &parsed_input
                        .interceptors
                        .iter()
                        .filter_map(|interceptor| match interceptor {
                            crate::controllers::web::InterceptorArgs::SwordSimple(path) => {
                                Some(path.clone())
                            }
                            crate::controllers::web::InterceptorArgs::SwordWithConfig {
                                middleware,
                                ..
                            } => Some(middleware.clone()),
                            crate::controllers::web::InterceptorArgs::Expression(_) => None,
                        })
                        .collect::<Vec<_>>(),
                )
            }
        }
        ParsedControllerKind::Grpc => {
            return Err(syn::Error::new(
                input.ident.span(),
                "Controller::Grpc is not implemented yet",
            ));
        }
    };

    let expanded = quote! {
        #input
        #builder
    };

    Ok(TokenStream::from(expanded))
}
