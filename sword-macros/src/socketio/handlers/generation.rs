use super::parsing::{CategorizedHandlers, HandlerInfo};
use proc_macro2::TokenStream;
use quote::quote;
use syn::{Ident, Type};

/// Generates the complete SocketIO adapter implementation.
/// Creates the setup function and adapter trait implementation for the given struct.
pub fn generate_socketio_handlers(
    struct_ty: &Type,
    categorized: CategorizedHandlers,
) -> syn::Result<TokenStream> {
    let socketio_namespace = quote! { <#struct_ty as ::sword::web::socketio::SocketIoAdapter>::namespace() };

    let connection_handler_code = categorized
        .on_connection
        .map(generate_connection_handler)
        .transpose()?;

    let message_handler_codes: Vec<TokenStream> = categorized
        .message_handlers
        .iter()
        .map(|(event_name, handler)| generate_message_handler(event_name, handler))
        .collect::<syn::Result<Vec<_>>>()?;

    let disconnection_handler_code = categorized
        .on_disconnection
        .map(generate_disconnection_handler)
        .transpose()?;

    let fallback_handler_code = categorized
        .on_fallback
        .map(generate_fallback_handler)
        .transpose()?;

    Ok(quote! {
        impl #struct_ty {
            #[doc(hidden)]
            pub fn __socketio_setup(state: &::sword::core::State) {
                let adapter = std::sync::Arc::new(
                    <#struct_ty as ::sword::core::Build>::build(state).unwrap_or_else(|err| {
                        panic!(
                            "\n[!] Failed to build {} SocketIO adapter\n\n{}\n",
                            stringify!(#struct_ty),
                            err
                        )
                    })
                );

                let io = <::sword::web::socketio::SocketIo as ::sword::core::FromState>::from_state(state)
                    .unwrap_or_else(|_| {
                        panic!("\n[!] SocketIo component not found in state. Is SocketIo correctly configured?\n\n   ↳ Ensure that the `socketio` feature is enabled in your `Cargo.toml`.\n   ↳ Also, make sure to configure the `socketio` server in your configuration file.\n   ↳ See the Sword documentation for more details: https://sword-web.github.io\n")
                    });

                io.ns(#socketio_namespace, move |socket: ::sword::web::socketio::SocketRef| {
                    let adapter_for_handler = adapter.clone();

                    async move {
                        #connection_handler_code
                        #(#message_handler_codes)*
                        #fallback_handler_code
                        #disconnection_handler_code
                    }
                });
            }
        }

        impl ::sword::core::Adapter for #struct_ty
        where
            Self: ::sword::web::socketio::SocketIoAdapter
        {
            fn kind() -> ::sword::core::AdapterKind {
                ::sword::core::AdapterKind::SocketIo(Box::new(Self::__socketio_setup))
            }
        }
    })
}

/// Generates code for the `#[on_connection]` handler.
/// This handler runs when a client connects to the socket.
fn generate_connection_handler(handler: &HandlerInfo) -> syn::Result<TokenStream> {
    let handler_name = &handler.name;
    let params = HandlerParams::from_args(&handler.args);
    let call_params = params.build_call_params();

    Ok(quote! {
        adapter_for_handler.#handler_name(#(#call_params),*).await;
    })
}

/// Generates code for a `#[on_message("event_name")]` handler.
/// Creates a listener for specific socket events with the given name.
fn generate_message_handler(
    event_name: &str,
    handler: &HandlerInfo,
) -> syn::Result<TokenStream> {
    let handler_name = &handler.name;
    let params = HandlerParams::from_args(&handler.args);

    let closure_params = params.build_closure_params();
    let call_params = params.build_call_params();

    Ok(quote! {
        socket.on(#event_name, {
            let adapter = adapter_for_handler.clone();
            async move |#closure_params| {
                adapter.#handler_name(#(#call_params),*).await;
            }
        });
    })
}

/// Generates code for the `#[on_disconnection]` handler.
/// This handler runs when a client disconnects from the socket.
fn generate_disconnection_handler(
    handler: &HandlerInfo,
) -> syn::Result<TokenStream> {
    let handler_name = &handler.name;
    let params = HandlerParams::from_args(&handler.args);
    let call_params = params.build_call_params();

    Ok(quote! {
        socket.on_disconnect({
            let adapter = adapter_for_handler.clone();
            async move || {
                adapter.#handler_name(#(#call_params),*).await;
            }
        });
    })
}

/// Generates code for the `#[on_fallback]` handler.
/// This handler runs for any event that doesn't match other handlers.
fn generate_fallback_handler(handler: &HandlerInfo) -> syn::Result<TokenStream> {
    let handler_name = &handler.name;
    let params = HandlerParams::from_args(&handler.args);

    let closure_params = params.build_closure_params();
    let call_params = params.build_call_params();

    Ok(quote! {
        socket.on_fallback({
            let adapter = adapter_for_handler.clone();
            async move |#closure_params| {
                adapter.#handler_name(#(#call_params),*).await;
            }
        });
    })
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
            .map(|(i, (_, _ty))| {
                let var_name = quote::format_ident!("p{}", i);
                quote! { #var_name }
            })
            .collect()
    }
}
