macro_rules! http_method {
    ($($method:ident => $method_str:expr),+ $(,)?) => {
        $(
            #[proc_macro_attribute]
            pub fn $method(attr: proc_macro::TokenStream, item: proc_macro::TokenStream) -> proc_macro::TokenStream {
                use crate::shared::CMetaStack;
                use syn::{parse_macro_input, ItemFn, LitStr};
                use quote::quote;

                // Parse the path from attribute
                let path = if attr.is_empty() {
                    String::from("/")
                } else {
                    parse_macro_input!(attr as LitStr).value()
                };

                let mut input_fn = parse_macro_input!(item as ItemFn);
                let fn_name = input_fn.sig.ident.clone();

                // === Parse interceptor attributes from the function ===

                use crate::adapters::{InterceptorArgs, expand_interceptor_args};

                let mut interceptors = Vec::new();
                let mut retained_attrs = Vec::new();

                for attr in input_fn.attrs.iter() {
                    if attr.path().is_ident("interceptor") {
                        // Parse interceptor attribute
                        match attr.parse_args::<InterceptorArgs>() {
                            Ok(args) => interceptors.push(args),
                            Err(e) => {
                                return syn::Error::new_spanned(attr, format!("Failed to parse interceptor: {}", e))
                                    .to_compile_error()
                                    .into();
                            }
                        }
                    } else {
                        retained_attrs.push(attr.clone());
                    }
                }

                // Remove interceptor attributes from the original function
                input_fn.attrs = retained_attrs;

                // === Read CMeta to get controller context ===

                let controller_name = CMetaStack::get("controller_name")
                    .unwrap_or_else(|| {
                        panic!(
                            "\n[ERROR] #[{}] must be used inside a #[controller] impl block.\n\
                             \n\
                             Make sure:\n\
                             1. The struct has #[controller(\"/path\")] attribute\n\
                             2. The struct is defined BEFORE the impl block\n\
                             3. The impl block is for the same struct\n\
                             \n\
                             Example:\n\
                             #[controller(\"/api\")]\n\
                             struct MyController {{}}\n\
                             \n\
                             impl MyController {{\n\
                                 #[{}(\"/\")]\n\
                                 async fn handler(&self) -> HttpResult {{ Ok(()) }}\n\
                             }}\n",
                            stringify!($method),
                            stringify!($method)
                        )
                    });

                let controller_path = CMetaStack::get("controller_path")
                    .unwrap_or_default();

                // Convert controller_name String to syn::Ident
                let controller_ident = quote::format_ident!("{}", controller_name);

                // === Generate __sword_route_{name} helper method ===

                let route_fn_name = quote::format_ident!("__sword_route_{}", fn_name);

                // Parse function parameters to generate closure
                let params: Vec<_> = input_fn.sig.inputs.iter()
                    .filter_map(|arg| {
                        if let syn::FnArg::Typed(pat_type) = arg {
                            Some((pat_type.pat.clone(), pat_type.ty.clone()))
                        } else {
                            None // Skip &self
                        }
                    })
                    .collect();

                let routing_fn = match $method_str {
                    "GET" => quote! { get },
                    "POST" => quote! { post },
                    "PUT" => quote! { put },
                    "DELETE" => quote! { delete },
                    "PATCH" => quote! { patch },
                    _ => quote! { get },
                };

                let mut handler = if params.is_empty() {
                    // No parameters (except &self)
                    quote! {
                        ::sword::internal::axum::routing::#routing_fn({
                            let ctrl = std::sync::Arc::clone(&controller);
                            move || async move {
                                use ::sword::internal::axum::IntoResponse;
                                ctrl.#fn_name().await.into_response()
                            }
                        })
                    }
                } else {
                    // Has parameters - generate closure with typed params
                    let closure_params: Vec<_> = params.iter()
                        .enumerate()
                        .map(|(i, (_, ty))| {
                            let param_name = quote::format_ident!("p{}", i);
                            quote! { #param_name: #ty }
                        })
                        .collect();

                    let call_args: Vec<_> = (0..params.len())
                        .map(|i| {
                            let param_name = quote::format_ident!("p{}", i);
                            quote! { #param_name }
                        })
                        .collect();

                    quote! {
                        ::sword::internal::axum::routing::#routing_fn({
                            let ctrl = std::sync::Arc::clone(&controller);
                            move |#(#closure_params),*| async move {
                                use ::sword::internal::axum::IntoResponse;
                                ctrl.#fn_name(#(#call_args),*).await.into_response()
                            }
                        })
                    }
                };

                // === Apply interceptors as layers (in reverse order) ===

                for interceptor in interceptors.iter().rev() {
                    let generated_interceptor = expand_interceptor_args(interceptor);
                    handler = quote! {
                        #handler.layer(#generated_interceptor)
                    };
                }

                // Generate inventory registration for this route
                // The closure captures the controller type and method at compile time
                // Use a unique name based on controller and method to avoid collisions
                let registration_name = quote::format_ident!(
                    "__SWORD_ROUTE_REGISTRAR_{}_{}",
                    controller_name.replace("::", "_"),
                    fn_name
                );

                let inventory_registration = quote! {
                    #[allow(non_upper_case_globals)]
                    #[doc(hidden)]
                    const #registration_name: () = {
                        ::sword::internal::inventory::submit! {
                            ::sword::internal::rest::RouteRegistrar::new(
                                ::std::any::TypeId::of::<#controller_ident>(),
                                #controller_name,
                                #controller_path,
                                #path,
                                |state: ::sword::internal::core::State| -> ::sword::internal::axum::MethodRouter<::sword::internal::core::State> {
                                    let controller = std::sync::Arc::new(
                                        #controller_ident::build(&state).unwrap_or_else(|err| {
                                            panic!("\n[!] Failed to build controller '{}'\n\n{}\n", #controller_name, err)
                                        })
                                    );
                                    #controller_ident::#route_fn_name(controller, state)
                                },
                                |router: ::sword::internal::axum::AxumRouter<::sword::internal::core::State>, state: ::sword::internal::core::State| -> ::sword::internal::axum::AxumRouter<::sword::internal::core::State> {
                                    #controller_ident::apply_interceptors(router, state)
                                }
                            )
                        }
                    };
                };

                let expanded = quote! {
                    #input_fn

                    // Generated helper method that returns MethodRouter<State>
                    pub fn #route_fn_name(
                        controller: std::sync::Arc<Self>,
                        state: ::sword::internal::core::State,
                    ) -> ::sword::internal::axum::MethodRouter<::sword::internal::core::State> {
                        #handler
                    }

                    // Auto-register this route via inventory
                    #inventory_registration
                };

                proc_macro::TokenStream::from(expanded)
            }
        )+
    };
}
