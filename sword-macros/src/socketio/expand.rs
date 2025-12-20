//! WebSocket macro expansion logic

use proc_macro::TokenStream;
use quote::quote;
use syn::{ImplItem, ItemImpl, ItemStruct, parse_macro_input};

use super::parsing::{HandlerType, WebSocketPath, get_handler_type};

/// Expands the `#[web_socket_gateway]` macro
pub fn expand_websocket_gateway(
    _attr: TokenStream,
    item: TokenStream,
) -> TokenStream {
    let input = parse_macro_input!(item as ItemStruct);
    let name = &input.ident;
    let vis = &input.vis;
    let fields = &input.fields;

    // Extract field names and types for Build implementation
    let field_inits = if let syn::Fields::Named(named_fields) = fields {
        named_fields.named.iter().map(|field| {
            let field_name = &field.ident;
            let field_type = &field.ty;
            quote! {
                #field_name: <#field_type as ::sword::core::FromStateArc>::from_state_arc(state)
                    .map_err(|_| ::sword::core::DependencyInjectionError::DependencyNotFound {
                        type_name: stringify!(#field_type).to_string(),
                    })?
            }
        }).collect::<Vec<_>>()
    } else {
        vec![]
    };

    let expanded = quote! {
        #[derive(Clone)]
        #vis struct #name #fields

        impl #name {
            pub fn router(state: ::sword::core::State) -> ::sword::__internal::AxumRouter {
                ::sword::__internal::AxumRouter::new().with_state(state)
            }
        }

        impl ::sword::web::websocket::WebSocketGateway for #name {
            fn router(state: ::sword::core::State) -> ::sword::__internal::AxumRouter {
                Self::router(state)
            }
        }

        impl ::sword::core::Build for #name {
            type Error = ::sword::core::DependencyInjectionError;

            fn build(state: &::sword::core::State) -> Result<Self, Self::Error> {
                Ok(Self {
                    #(#field_inits),*
                })
            }
        }
    };

    expanded.into()
}

// Helper struct to store handler parameter information
struct HandlerParams {
    has_socket: bool,
    has_data: bool,
    has_event: bool,
    has_ack: bool,
}

// Helper function to analyze handler parameters
fn analyze_handler_params(sig: &syn::Signature) -> HandlerParams {
    let mut params = HandlerParams {
        has_socket: false,
        has_data: false,
        has_event: false,
        has_ack: false,
    };

    for arg in &sig.inputs {
        if let syn::FnArg::Typed(pat_type) = arg {
            let ty_str = quote::quote!(#pat_type.ty).to_string();
            if ty_str.contains("SocketRef") {
                params.has_socket = true;
            }
            if ty_str.contains("Data") {
                params.has_data = true;
            }
            if ty_str.contains("Event") {
                params.has_event = true;
            }
            if ty_str.contains("AckSender") {
                params.has_ack = true;
            }
        }
    }

    params
}

/// Expands the `#[web_socket]` macro
pub fn expand_websocket(attr: TokenStream, item: TokenStream) -> TokenStream {
    let path_struct = parse_macro_input!(attr as WebSocketPath);
    let path = path_struct.path;
    let input = parse_macro_input!(item as ItemImpl);

    let self_ty = &input.self_ty;

    // Extract all handler methods from the impl block with their parameter info
    let mut on_connect_handler: Option<(syn::Ident, HandlerParams)> = None;
    let mut message_handlers: Vec<(String, syn::Ident, HandlerParams)> = Vec::new();
    let mut on_disconnect_handler: Option<(syn::Ident, HandlerParams)> = None;
    let mut on_fallback_handler: Option<(syn::Ident, HandlerParams)> = None;

    for item in &input.items {
        if let ImplItem::Fn(method) = item {
            if let Some((handler_type, message_type)) =
                get_handler_type(&method.attrs)
            {
                let method_name = method.sig.ident.clone();
                let params = analyze_handler_params(&method.sig);

                match handler_type {
                    HandlerType::OnConnection => {
                        on_connect_handler = Some((method_name, params));
                    }
                    HandlerType::OnDisconnect => {
                        on_disconnect_handler = Some((method_name, params));
                    }
                    HandlerType::OnFallback => {
                        on_fallback_handler = Some((method_name, params));
                    }
                    HandlerType::SubscribeMessage => {
                        let msg_name =
                            message_type.unwrap_or_else(|| "message".to_string());
                        message_handlers.push((msg_name, method_name, params));
                    }
                }
            }
        }
    }

    // Generate message handler codes that call the actual methods
    // Note: For now we skip message handlers due to lifetime constraints
    // Message handlers would need to be registered outside the socket setup
    #[allow(unused)]
    let message_handler_codes: Vec<proc_macro2::TokenStream> = vec![];

    // Generate message handler registration code
    let message_handler_registrations = message_handlers.iter().map(|(event_name, method_name, params)| {
        // Build the closure parameters based on what the handler expects
        let closure_params = if params.has_socket && params.has_data && params.has_ack {
            quote! { socket: sword::prelude::websocket::SocketRef,
                     data: sword::prelude::websocket::Data<sword::prelude::websocket::Value>,
                     ack: sword::prelude::websocket::AckSender }
        } else if params.has_socket && params.has_data {
            quote! { socket: sword::prelude::websocket::SocketRef,
                     data: sword::prelude::websocket::Data<sword::prelude::websocket::Value> }
        } else if params.has_data && params.has_ack {
            quote! { _socket: sword::prelude::websocket::SocketRef,
                     data: sword::prelude::websocket::Data<sword::prelude::websocket::Value>,
                     ack: sword::prelude::websocket::AckSender }
        } else if params.has_socket && params.has_ack {
            quote! { socket: sword::prelude::websocket::SocketRef,
                     _data: sword::prelude::websocket::Data<sword::prelude::websocket::Value>,
                     ack: sword::prelude::websocket::AckSender }
        } else if params.has_data {
            quote! { _socket: sword::prelude::websocket::SocketRef,
                     data: sword::prelude::websocket::Data<sword::prelude::websocket::Value> }
        } else if params.has_socket {
            quote! { socket: sword::prelude::websocket::SocketRef,
                     _data: sword::prelude::websocket::Data<sword::prelude::websocket::Value> }
        } else if params.has_ack {
            quote! { _socket: sword::prelude::websocket::SocketRef,
                     _data: sword::prelude::websocket::Data<sword::prelude::websocket::Value>,
                     ack: sword::prelude::websocket::AckSender }
        } else {
            quote! { _socket: sword::prelude::websocket::SocketRef,
                     _data: sword::prelude::websocket::Data<sword::prelude::websocket::Value> }
        };

        // Build the method call parameters based on what the handler expects
        // Standard order: SocketRef, Event, Data, AckSender
        let mut call_params = vec![];
        if params.has_socket {
            call_params.push(quote! { socket });
        }
        if params.has_event {
            call_params.push(quote! { sword::prelude::websocket::Event(#event_name.to_string()) });
        }
        if params.has_data {
            call_params.push(quote! { data });
        }
        if params.has_ack {
            call_params.push(quote! { ack });
        }

        quote! {
            {
                let state_for_msg = state_for_handler.clone();
                socket.on(#event_name, move |#closure_params| {
                    let state = state_for_msg.clone();
                    async move {
                        match <std::sync::Arc<#self_ty> as ::sword::core::FromStateArc>::from_state_arc(&state) {
                            Ok(controller) => {
                                controller.#method_name(#(#call_params),*).await;
                            }
                            Err(e) => {
                                sword::__internal::tracing::error!("Failed to instantiate controller for message handler: {}", e);
                            }
                        }
                    }
                });
            }
        }
    }).collect::<Vec<_>>();

    // Generate disconnect handler registration code
    let disconnect_handler_registration = if let Some((disconnect_method, params)) =
        on_disconnect_handler.as_ref()
    {
        let call_params = if params.has_socket {
            quote! { sock }
        } else {
            quote! {}
        };

        quote! {
            {
                let state_for_disconnect = state_for_handler.clone();
                let socket_for_disconnect = socket.clone();
                socket.on_disconnect(move || {
                    let state = state_for_disconnect.clone();
                    let sock = socket_for_disconnect.clone();
                    async move {
                        match <std::sync::Arc<#self_ty> as ::sword::core::FromStateArc>::from_state_arc(&state) {
                            Ok(controller) => {
                                controller.#disconnect_method(#call_params).await;
                            }
                            Err(e) => {
                                sword::__internal::tracing::error!("Failed to instantiate controller for disconnect handler: {}", e);
                            }
                        }
                    }
                });
            }
        }
    } else {
        quote! {}
    };

    // Generate fallback handler registration code (catch-all for unhandled events)
    let fallback_handler_registration = if let Some((fallback_method, params)) =
        on_fallback_handler.as_ref()
    {
        // Build the closure parameters based on what the handler expects
        let closure_params = if params.has_event && params.has_data {
            quote! { event: sword::prelude::websocket::Event,
            data: sword::prelude::websocket::Data<sword::prelude::websocket::Value> }
        } else if params.has_event {
            quote! { event: sword::prelude::websocket::Event,
            _data: sword::prelude::websocket::Data<sword::prelude::websocket::Value> }
        } else if params.has_data {
            quote! { _event: sword::prelude::websocket::Event,
            data: sword::prelude::websocket::Data<sword::prelude::websocket::Value> }
        } else {
            quote! { _event: sword::prelude::websocket::Event,
            _data: sword::prelude::websocket::Data<sword::prelude::websocket::Value> }
        };

        // Build the method call parameters
        let mut call_params = vec![];
        if params.has_event {
            call_params.push(quote! { event });
        }
        if params.has_data {
            call_params.push(quote! { data });
        }

        quote! {
            {
                let state_for_fallback = state_for_handler.clone();
                // Use socketioxide's on_fallback method for catch-all event handling
                socket.on_fallback(move |#closure_params| {
                    let state = state_for_fallback.clone();
                    async move {
                        match <std::sync::Arc<#self_ty> as ::sword::core::FromStateArc>::from_state_arc(&state) {
                            Ok(controller) => {
                                controller.#fallback_method(#(#call_params),*).await;
                            }
                            Err(e) => {
                                sword::__internal::tracing::error!("Failed to instantiate controller for fallback handler: {}", e);
                            }
                        }
                    }
                });
            }
        }
    } else {
        quote! {}
    };

    // Generate the handler call code
    let handler_call = if let Some((handler_name, params)) =
        on_connect_handler.as_ref()
    {
        let call_params = if params.has_socket {
            quote! { socket.clone() }
        } else {
            quote! {}
        };

        quote! {
            match <std::sync::Arc<#self_ty> as ::sword::core::FromStateArc>::from_state_arc(&state_for_handler) {
                Ok(controller) => {
                    controller.#handler_name(#call_params).await;
                }
                Err(e) => {
                    sword::__internal::tracing::error!("Failed to instantiate controller: {}", e);
                }
            }
        }
    } else {
        quote! {
            sword::__internal::tracing::info!(ns = socket.ns(), ?socket.id, "Socket.IO connected");
        }
    };

    let expanded = quote! {
        #input

        impl ::sword::web::websocket::WebSocketProvider for #self_ty {
            fn path() -> &'static str {
                Box::leak(#path.into())
            }

            fn get_setup_fn(_: ::sword::core::State) -> ::sword::web::websocket::SocketSetupFn {
                std::sync::Arc::new(|io: &::sword::web::websocket::SocketIo, state_from_call: ::sword::core::State| {
                    let app_state = state_from_call.clone();  // Clone state BEFORE ns() call

                    io.ns(#path, move |socket: ::sword::web::websocket::SocketRef| {
                        let state_for_handler = app_state.clone();  // Clone again for the async block

                        // Run connection handler (async)
                        Box::pin(async move {
                            // Call on_connect handler with captured state
                            #handler_call

                            // Register message handlers
                            #(#message_handler_registrations)*

                            // Register fallback handler (must be before disconnect to catch all unhandled events)
                            #fallback_handler_registration

                            // Register disconnect handler
                            #disconnect_handler_registration
                        })
                    });
                })
            }

            fn router(state: ::sword::core::State) -> ::sword::__internal::AxumRouter {
                ::sword::__internal::AxumRouter::new().with_state(state)
            }
        }
    };

    expanded.into()
}
