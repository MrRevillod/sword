use super::parsing::{EventKind, HandlerInfo};
use proc_macro2::TokenStream;
use quote::quote;
use syn::{Ident, Path, Type};

/// Generates the complete SocketIO adapter implementation.
/// Creates the setup function and adapter trait implementation for the given struct.
pub fn generate_socketio_handlers(
    struct_ty: &Type,
    handlers: &[HandlerInfo],
) -> syn::Result<TokenStream> {
    let socketio_namespace = quote! {
        <#struct_ty as ::sword::internal::core::SocketIoAdapter>::namespace()
    };

    let common_handlers_code: Vec<TokenStream> = handlers
        .iter()
        .filter(|h| !matches!(h.event_kind, EventKind::OnConnection(_)))
        .map(generate_event_handler)
        .collect();

    let on_connection_handlers: Vec<&HandlerInfo> = handlers
        .iter()
        .filter(|h| matches!(h.event_kind, EventKind::OnConnection(_)))
        .collect();

    let connection_params = on_connection_handlers
        .first()
        .map(|h| HandlerParams::from_args(&h.args))
        .unwrap_or_else(|| HandlerParams { params: vec![] });

    let connection_closure_params = connection_params.build_closure_params();
    let on_connection_handler_code: Vec<TokenStream> = on_connection_handlers
        .iter()
        .map(|h| generate_connection_handler(h, &connection_params))
        .collect();

    let all_interceptors: Vec<&Path> = handlers
        .iter()
        .filter_map(|h| match &h.event_kind {
            EventKind::OnConnection(interceptors) => Some(interceptors.as_slice()),
            _ => None,
        })
        .flatten()
        .collect();

    let interceptor_applications = all_interceptors.iter().map(|interceptor_path| {
        quote! {
            let interceptor = state.borrow::<::std::sync::Arc<#interceptor_path>>()
                .expect(&format!("\n[!] Failed to retrieve interceptor {} from State\n", stringify!(#interceptor_path)));

            let handler = handler.with(move |ctx: ::sword::prelude::SocketContext| {
                let interceptor = ::std::sync::Arc::clone(&interceptor);
                async move {
                    <#interceptor_path as ::sword::prelude::OnConnect>::on_connect(&*interceptor, ctx)
                        .await
                        .map_err(|e| ::std::boxed::Box::new(e) as ::std::boxed::Box<dyn ::std::fmt::Display + Send>)
                }
            });
        }
    });

    let base_handler = quote! {
        let handler = move |socket: ::sword::prelude::SocketRef, #connection_closure_params| {
            let adapter = adapter.clone();

            async move {
                #(#on_connection_handler_code)*
                #(#common_handlers_code)*
            }
        };

        #(#interceptor_applications)*
    };

    Ok(quote! {
        impl #struct_ty {
            #[doc(hidden)]
            pub fn __socketio_setup(state: &::sword::internal::core::State) {
                use ::sword::internal::socketio::ConnectHandler;

                let adapter = std::sync::Arc::new(
                    <#struct_ty as ::sword::internal::core::Build>::build(state).unwrap_or_else(|err| {
                        panic!(
                            "\n[!] Failed to build {} SocketIO adapter\n\n{}\n",
                            stringify!(#struct_ty),
                            err
                        )
                    })
                );

                let io = <::sword::prelude::SocketIo as ::sword::internal::core::FromState>::from_state(state)
                    .expect("\n[!] SocketIo component not found in state. Is SocketIo correctly configured?\n\n   ↳ Ensure that the `socketio` feature is enabled in your `Cargo.toml`.\n   ↳ Also, make sure to configure the `socketio` server in your configuration file.\n   ↳ See the Sword documentation for more details: https://sword-web.github.io\n");

                #base_handler

                io.ns(#socketio_namespace, handler);
            }
        }

        impl ::sword::internal::core::Adapter for #struct_ty {
            fn kind() -> ::sword::internal::core::AdapterKind {
                ::sword::internal::core::AdapterKind::SocketIo(Box::new(Self::__socketio_setup))
            }
        }
    })
}

/// Generates code for the `#[on_connection]` handler.
/// This handler runs when a client connects to the socket.
/// Receives parameters from the closure and passes them to the adapter method.
fn generate_connection_handler(
    handler: &HandlerInfo,
    params: &HandlerParams,
) -> TokenStream {
    let handler_name = &handler.name;
    let call_params = params.build_call_params();

    quote! {
        adapter.#handler_name(#(#call_params),*).await;
    }
}

/// Generates code for every `EventKind` except connection handlers.
/// Creates a listener for specific socket events with the given name.
fn generate_event_handler(handler: &HandlerInfo) -> TokenStream {
    let handler_name = &handler.name;
    let params = HandlerParams::from_args(&handler.args);

    let call_params = params.build_call_params();
    let closure_params = params.build_closure_params();

    let inner_handler = quote! {
        let adapter = adapter.clone();
        async move |#closure_params| {
            adapter.#handler_name(#(#call_params),*).await;
        }
    };

    if let EventKind::OnMessage(event_name) = &handler.event_kind {
        return quote! {
            socket.on(#event_name, { #inner_handler });
        };
    }

    if let EventKind::OnDisconnection = &handler.event_kind {
        return quote! {
            socket.on_disconnect({ #inner_handler });
        };
    }

    if let EventKind::Fallback = &handler.event_kind {
        return quote! {
            socket.on_fallback({ #inner_handler });
        };
    }

    quote! {}
}

/// Stores parameters from handler function for code generation.
/// Parameters excluding `&self`, stored as (variable_name, type)
struct HandlerParams {
    params: Vec<(Ident, Type)>,
}

impl HandlerParams {
    fn from_args(args: &[(Ident, Type)]) -> Self {
        Self {
            params: args.to_vec(),
        }
    }

    /// Generates closure parameter list for socket.on() handlers.
    /// Creates simple variable names (p0, p1, p2...) to receive values from socketioxide.
    fn build_closure_params(&self) -> TokenStream {
        let params: Vec<_> = self
            .params
            .iter()
            .enumerate()
            .map(|(i, (_, ty))| {
                let var_name = quote::format_ident!("p{}", i);
                quote! { #var_name: #ty }
            })
            .collect();

        quote! { #(#params),* }
    }

    /// Generates parameters to pass to the user's handler method.
    /// Simply passes closure variables (p0, p1, p2...) directly to the handler.
    fn build_call_params(&self) -> Vec<TokenStream> {
        self.params
            .iter()
            .enumerate()
            .map(|(i, (_, _))| {
                let var_name = quote::format_ident!("p{}", i);
                quote! { #var_name }
            })
            .collect()
    }
}
