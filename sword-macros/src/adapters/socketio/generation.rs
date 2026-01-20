use crate::shared::{CommonHttpAdapterInput, gen_build, gen_clone, gen_deps};

use proc_macro2::TokenStream;
use quote::quote;
use syn::Path;

pub fn generate_socketio_adapter_builder(
    input: &CommonHttpAdapterInput,
    interceptors: &[Path],
) -> TokenStream {
    let namespace = &input.base_path;
    let self_name = &input.struct_name;
    let self_fields = &input.fields;
    let adapter_name_str = self_name.to_string();

    let deps_impl = gen_deps(self_name, self_fields);
    let build_impl = gen_build(self_name, self_fields);
    let clone_impl = gen_clone(self_name, self_fields);

    let interceptor_applications = interceptors.iter().map(|interceptor_path| {
        quote! {
            let interceptor = ::std::sync::Arc::new(
                state.borrow::<#interceptor_path>()
                    .expect(&format!("\n[!] Failed to retrieve interceptor {} from State\n", stringify!(#interceptor_path)))
                    .clone()
            );

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

    let setup_impl = quote! {
        #[doc(hidden)]
        pub fn __socketio_setup(state: &::sword::internal::core::State) {
            use ::sword::internal::socketio::ConnectHandler;

            let adapter = ::std::sync::Arc::new(
                <#self_name as ::sword::internal::core::Build>::build(state).unwrap_or_else(|err| {
                    eprintln!("\n[!] Failed to build SocketIO adapter: {}\n", #adapter_name_str);
                    eprintln!("    Error: {}\n", err);
                    eprintln!("    This usually means a dependency is missing from the State.");
                    eprintln!("    Ensure all dependencies are registered as providers or components.\n");
                    panic!("SocketIO adapter build failed");
                })
            );

            let io = <::sword::prelude::SocketIo as ::sword::internal::core::FromState>::from_state(state)
                .expect("\n[!] SocketIo component not found in state. Is SocketIo correctly configured?\n\n   ↳ Ensure that the `socketio` feature is enabled in your `Cargo.toml`.\n   ↳ Also, make sure to configure the `socketio` server in your configuration file.\n   ↳ See the Sword documentation for more details: https://sword-web.github.io\n");

            let base_handler = {
                let adapter_clone = ::std::sync::Arc::clone(&adapter);

                move |ctx: ::sword::prelude::SocketContext| -> ::std::pin::Pin<::std::boxed::Box<dyn ::std::future::Future<Output = ()> + ::std::marker::Send>> {
                    let adapter = ::std::sync::Arc::clone(&adapter_clone);
                    let socket = ctx.socket.clone();

                    ::std::boxed::Box::pin(async move {
                        for handler_meta in ::sword::internal::inventory::iter::<::sword::internal::socketio::HandlerRegistrar>() {
                            if handler_meta.adapter_type_id != ::std::any::TypeId::of::<#self_name>() {
                                continue;
                            }

                            match handler_meta.event_kind {
                                ::sword::internal::socketio::SocketEventKind::Connection => {
                                    let adapter_clone = ::std::sync::Arc::clone(&adapter);
                                    (handler_meta.call_fn)(adapter_clone, ctx).await;
                                    break;
                                },
                                ::sword::internal::socketio::SocketEventKind::Message(_)
                                    | ::sword::internal::socketio::SocketEventKind::Disconnection
                                    | ::sword::internal::socketio::SocketEventKind::Fallback => {
                                    let adapter_clone = ::std::sync::Arc::clone(&adapter);
                                    (handler_meta.register_fn)(adapter_clone, socket.clone());
                                }
                            }
                        }
                    })
                }
            };

            let handler = base_handler;
            #(#interceptor_applications)*

            io.ns(#namespace, handler);
        }
    };

    let setup_registration = quote! {
        const _: () = {
            ::sword::internal::inventory::submit! {
                ::sword::internal::socketio::SocketIoSetupFn {
                    adapter_type_id: ::std::any::TypeId::of::<#self_name>(),
                    setup: #self_name::__socketio_setup,
                }
            }
        };
    };

    quote! {
        #build_impl
        #deps_impl
        #clone_impl

        impl ::sword::internal::core::SocketIoAdapter for #self_name {
            fn namespace() -> &'static str {
                #namespace
            }
        }

        impl ::sword::internal::core::Adapter for #self_name {
            fn kind() -> ::sword::internal::core::AdapterKind {
                ::sword::internal::core::AdapterKind::SocketIo
            }

            fn type_id() -> ::std::any::TypeId {
                ::std::any::TypeId::of::<Self>()
            }
        }

        impl #self_name {
            #setup_impl
        }

        #setup_registration
    }
}
