use proc_macro::TokenStream;
use quote::quote;
use syn::ItemStruct;

#[cfg(feature = "socketio-controllers")]
use crate::common::{SocketIoControllerContext, push_socketio_controller_context};

#[cfg(feature = "socketio-controllers")]
use crate::controller::socketio::generate_socketio_controller_builder;
use crate::{
    common::{WebControllerContext, push_web_controller_context},
    controller::web::generate_controller_builder,
    controller::{CommonControllerInput, ParsedControllerKind},
    interceptor::InterceptorArgs,
};

fn has_sword_interceptors(interceptors: &[InterceptorArgs]) -> bool {
    interceptors.iter().any(|interceptor| {
        matches!(
            interceptor,
            InterceptorArgs::SwordSimple(_)
                | InterceptorArgs::SwordWithConfig { .. }
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
            push_web_controller_context(WebControllerContext {
                name: &struct_name,
                path: mount_path,
                has_sword_interceptors: has_sword_interceptors(
                    &parsed_input.interceptors,
                ),
            });

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
                push_socketio_controller_context(SocketIoControllerContext {
                    name: &struct_name,
                    namespace: mount_path,
                });

                generate_socketio_controller_builder(
                    &parsed_input,
                    &parsed_input.interceptors,
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
