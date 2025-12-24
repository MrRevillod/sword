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
                            "\n❌ Failed to build {} SocketIO adapter\n\n{}\n",
                            stringify!(#struct_ty),
                            err
                        )
                    })
                );

                let io = <::sword::web::socketio::SocketIo as ::sword::core::FromState>::from_state(state)
                    .unwrap_or_else(|_| {
                        panic!("\n❌ SocketIo component not found in state. Is SocketIo correctly configured?\n")
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
    let params = HandlerParams::analyze(&handler.args)?;
    let call_params = params.build_call_params(None, false);

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
    let params = HandlerParams::analyze(&handler.args)?;

    let closure_params = params.build_closure_params();
    let call_params = params.build_call_params(Some(event_name), false);

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
    let params = HandlerParams::analyze(&handler.args)?;
    let call_params = params.build_call_params(None, true);

    Ok(quote! {
        socket.on_disconnect({
            let adapter = adapter_for_handler.clone();
            let socket = socket.clone();
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
    let params = HandlerParams::analyze(&handler.args)?;

    let closure_params = params.build_fallback_closure_params();
    let call_params = params.build_fallback_call_params();

    Ok(quote! {
        socket.on_fallback({
            let adapter = adapter_for_handler.clone();
            async move |#(#closure_params),*| {
                adapter.#handler_name(#(#call_params),*).await;
            }
        });
    })
}

/// Represents the different types of parameters a socket handler can receive.
#[derive(Clone)]
enum ParamKind {
    Socket,
    Data(Type),
    TryData(Type),
    Event,
    Ack,
}

/// Stores analyzed parameters from a handler function signature.
struct HandlerParams {
    params: Vec<(Ident, ParamKind)>,
}

impl HandlerParams {
    /// Analyzes handler function arguments and identifies their types.
    /// Returns error if Data<T> or TryData<T> don't have a generic type.
    fn analyze(args: &[(Ident, Type)]) -> syn::Result<Self> {
        let mut params = Vec::new();

        for (ident, ty) in args {
            let ty_str = quote::quote!(#ty).to_string();

            let kind = match () {
                _ if ty_str.contains("SocketRef") => ParamKind::Socket,
                _ if ty_str.contains("TryData") => {
                    ParamKind::TryData(Self::extract_generic_type(ty)?)
                }
                _ if ty_str.contains("Data") => {
                    ParamKind::Data(Self::extract_generic_type(ty)?)
                }
                _ if ty_str.contains("Event") => ParamKind::Event,
                _ if ty_str.contains("AckSender") => ParamKind::Ack,
                _ => continue,
            };

            params.push((ident.clone(), kind));
        }

        Ok(Self { params })
    }

    /// Extracts the inner generic type from Data<T> or TryData<T>.
    /// Returns error if the type is not properly formed.
    fn extract_generic_type(ty: &Type) -> syn::Result<Type> {
        let Type::Path(type_path) = ty else {
            return Err(syn::Error::new_spanned(
                ty,
                "Data<T> and TryData<T> must have an explicit generic type parameter",
            ));
        };

        let Some(segment) = type_path.path.segments.last() else {
            return Err(syn::Error::new_spanned(
                ty,
                "Data<T> and TryData<T> must have an explicit generic type parameter",
            ));
        };

        let syn::PathArguments::AngleBracketed(args) = &segment.arguments else {
            return Err(syn::Error::new_spanned(
                ty,
                "Data<T> and TryData<T> must have an explicit generic type parameter",
            ));
        };

        let Some(syn::GenericArgument::Type(inner_ty)) = args.args.first() else {
            return Err(syn::Error::new_spanned(
                ty,
                "Data<T> and TryData<T> must have an explicit generic type parameter",
            ));
        };

        Ok(inner_ty.clone())
    }

    /// Checks if the handler has a SocketRef parameter.
    fn has_socket(&self) -> bool {
        self.params
            .iter()
            .any(|(_, k)| matches!(k, ParamKind::Socket))
    }

    /// Checks if the handler has an Event parameter.
    fn has_event(&self) -> bool {
        self.params
            .iter()
            .any(|(_, k)| matches!(k, ParamKind::Event))
    }

    /// Checks if the handler has an AckSender parameter.
    fn has_ack(&self) -> bool {
        self.params.iter().any(|(_, k)| matches!(k, ParamKind::Ack))
    }

    /// Returns the Data<T> or TryData<T> type if present in the handler.
    fn data_type(&self) -> Option<TokenStream> {
        self.params.iter().find_map(|(_, kind)| match kind {
            ParamKind::TryData(inner_ty) => {
                Some(quote! { ::sword::web::socketio::TryData<#inner_ty> })
            }
            ParamKind::Data(inner_ty) => {
                Some(quote! { ::sword::web::socketio::Data<#inner_ty> })
            }
            _ => None,
        })
    }

    /// Builds the closure parameter list for socket.on() callbacks.
    /// Only includes parameters that the user specified in their handler.
    fn build_closure_params(&self) -> TokenStream {
        let mut params = Vec::new();

        if self.has_socket() {
            params.push(quote! { socket: ::sword::web::socketio::SocketRef });
        }

        if let Some(data_type) = self.data_type() {
            params.push(quote! { data: #data_type });
        }

        if self.has_ack() {
            params.push(quote! { ack: ::sword::web::socketio::AckSender });
        }

        quote! { #(#params),* }
    }

    /// Builds the arguments list when calling the user's handler function.
    /// Preserves the order specified by the user in their function signature.
    fn build_call_params(
        &self,
        event_name: Option<&str>,
        use_cloned_socket: bool,
    ) -> Vec<TokenStream> {
        self.params
            .iter()
            .filter_map(|(_, kind)| match kind {
                ParamKind::Socket => Some(if use_cloned_socket {
                    quote! { socket }
                } else {
                    quote! { socket.clone() }
                }),
                ParamKind::Event => event_name.map(|name| {
                    quote! { ::sword::web::socketio::Event(#name.to_string()) }
                }),
                ParamKind::Data(_) | ParamKind::TryData(_) => Some(quote! { data }),
                ParamKind::Ack => Some(quote! { ack }),
            })
            .collect()
    }

    /// Builds the arguments list for fallback handler calls.
    fn build_fallback_call_params(&self) -> Vec<TokenStream> {
        self.params
            .iter()
            .filter_map(|(_, kind)| match kind {
                ParamKind::Event => Some(quote! { event }),
                ParamKind::Data(_) | ParamKind::TryData(_) => Some(quote! { data }),
                _ => None,
            })
            .collect()
    }

    /// Builds the closure parameter list for socket.on_fallback() callback.
    fn build_fallback_closure_params(&self) -> Vec<TokenStream> {
        let mut params = Vec::new();

        if self.has_event() {
            params.push(quote! { event: ::sword::web::socketio::Event });
        }

        if let Some(data_type) = self.data_type() {
            params.push(quote! { data: #data_type });
        }

        params
    }
}
