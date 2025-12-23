use super::parsing::{CategorizedHandlers, HandlerInfo};
use proc_macro2::TokenStream;
use quote::quote;
use syn::{Ident, Type};

pub fn generate_socketio_handlers(
    struct_ty: &Type,
    categorized: CategorizedHandlers,
) -> syn::Result<TokenStream> {
    let path = quote! { <#struct_ty as ::sword::web::SocketIoAdapter>::path() };

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

                let io = <::sword::web::SocketIo as ::sword::core::FromState>::from_state(state)
                    .unwrap_or_else(|_| {
                        panic!("\n❌ SocketIo component not found in state. Is SocketIo correctly configured?\n")
                    });

                io.ns(#path, move |socket: ::sword::web::SocketRef| {
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
            Self: ::sword::web::SocketIoAdapter
        {
            fn kind() -> ::sword::core::AdapterKind {
                ::sword::core::AdapterKind::WebSocket(Box::new(Self::__socketio_setup))
            }
        }
    })
}

fn generate_connection_handler(handler: &HandlerInfo) -> syn::Result<TokenStream> {
    let handler_name = &handler.name;
    let params = HandlerParams::analyze(&handler.args);
    let call_params = params.build_call_params(None, false);

    Ok(quote! {
        adapter_for_handler.#handler_name(#(#call_params),*).await;
    })
}

fn generate_message_handler(
    event_name: &str,
    handler: &HandlerInfo,
) -> syn::Result<TokenStream> {
    let handler_name = &handler.name;
    let params = HandlerParams::analyze(&handler.args);

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

fn generate_disconnection_handler(
    handler: &HandlerInfo,
) -> syn::Result<TokenStream> {
    let handler_name = &handler.name;
    let params = HandlerParams::analyze(&handler.args);
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

fn generate_fallback_handler(handler: &HandlerInfo) -> syn::Result<TokenStream> {
    let handler_name = &handler.name;
    let params = HandlerParams::analyze(&handler.args);

    let closure_params = build_fallback_closure_params(&params);
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

fn build_fallback_closure_params(params: &HandlerParams) -> Vec<TokenStream> {
    let data_type = params.data_type();
    let has_data = params.has_any_data();

    let event_param = if params.has_event {
        quote! { event: ::sword::web::Event }
    } else {
        quote! { _event: ::sword::web::Event }
    };

    let data_param = if has_data {
        quote! { data: #data_type }
    } else {
        quote! { _data: #data_type }
    };

    vec![event_param, data_param]
}

#[derive(Debug)]
struct HandlerParams {
    has_socket: bool,
    has_data: bool,
    has_try_data: bool,
    has_event: bool,
    has_ack: bool,
}

impl HandlerParams {
    fn analyze(args: &[(Ident, Type)]) -> Self {
        let mut params = Self {
            has_socket: false,
            has_data: false,
            has_try_data: false,
            has_event: false,
            has_ack: false,
        };

        for (_, ty) in args {
            let ty_str = quote::quote!(#ty).to_string();

            if ty_str.contains("SocketRef") {
                params.has_socket = true;
            }
            if ty_str.contains("TryData") {
                params.has_try_data = true;
            } else if ty_str.contains("Data") {
                params.has_data = true;
            }
            if ty_str.contains("Event") {
                params.has_event = true;
            }
            if ty_str.contains("AckSender") {
                params.has_ack = true;
            }
        }

        params
    }

    fn has_any_data(&self) -> bool {
        self.has_data || self.has_try_data
    }

    fn data_type(&self) -> TokenStream {
        if self.has_try_data {
            quote! { ::sword::web::TryData<::sword::web::Value> }
        } else {
            quote! { ::sword::web::Data<::sword::web::Value> }
        }
    }

    fn build_closure_params(&self) -> TokenStream {
        let has_data = self.has_any_data();
        let data_type = self.data_type();

        let mut params = Vec::new();

        params.push(self.param_or_ignored(
            "socket",
            self.has_socket,
            quote! { ::sword::web::SocketRef },
        ));

        params.push(self.param_or_ignored("data", has_data, data_type));

        if self.has_ack {
            params.push(self.param_or_ignored(
                "ack",
                true,
                quote! { ::sword::web::AckSender },
            ));
        }

        quote! { #(#params),* }
    }

    fn build_call_params(
        &self,
        event_name: Option<&str>,
        use_cloned_socket: bool,
    ) -> Vec<TokenStream> {
        let mut params = Vec::new();

        if self.has_socket {
            if use_cloned_socket {
                params.push(quote! { socket });
            } else {
                params.push(quote! { socket.clone() });
            }
        }

        if self.has_event {
            if let Some(name) = event_name {
                params.push(quote! { ::sword::web::Event(#name.to_string()) });
            }
        }

        if self.has_any_data() {
            params.push(quote! { data });
        }

        if self.has_ack {
            params.push(quote! { ack });
        }

        params
    }

    fn build_fallback_call_params(&self) -> Vec<TokenStream> {
        let mut params = Vec::new();

        if self.has_event {
            params.push(quote! { event });
        }

        if self.has_any_data() {
            params.push(quote! { data });
        }

        params
    }

    fn param_or_ignored(
        &self,
        name: &str,
        used: bool,
        ty: TokenStream,
    ) -> TokenStream {
        let ident = if used {
            quote::format_ident!("{}", name)
        } else {
            quote::format_ident!("_{}", name)
        };

        quote! { #ident: #ty }
    }
}
