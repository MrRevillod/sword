use proc_macro::TokenStream;
use proc_macro_error::emit_error;
use quote::quote;
use syn::{Expr, ItemStruct, Lit, Meta, parse_macro_input};

pub fn expand_config_struct(attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemStruct);
    let meta = parse_macro_input!(attr as Meta);

    let toml_key_str = match meta {
        Meta::NameValue(nv) if nv.path.is_ident("key") => {
            if let Expr::Lit(expr) = nv.value {
                if let Lit::Str(lit_str) = expr.lit {
                    lit_str.value()
                } else {
                    emit_error!(expr, "Expected a string literal for the toml key");
                    return TokenStream::new();
                }
            } else {
                emit_error!(nv.value, "Expected a literal for the toml key");
                return TokenStream::new();
            }
        }
        _ => {
            emit_error!(meta, "Expected a `key = \"...\"` attribute");
            return TokenStream::new();
        }
    };

    let struct_name = &input.ident;

    let expanded = quote! {
        #input

        impl ::sword::core::ConfigItem for #struct_name {
            fn toml_key() -> &'static str {
                #toml_key_str
            }
        }

        // Implement TryFrom for use with #[injectable] macro
        // Config structs are extracted from the Config object in State,
        // not directly from State, so they use TryFrom instead of FromState.
        impl TryFrom<&::sword::core::State> for #struct_name {
            type Error = ::sword::core::DependencyInjectionError;

            fn try_from(state: &::sword::core::State) -> Result<Self, Self::Error> {
                let config = state.get::<::sword::core::Config>()
                    .map_err(|_| ::sword::core::DependencyInjectionError::DependencyNotFound {
                        type_name: "Config".to_string(),
                    })?;

                config.get::<Self>()
                    .map_err(|e| ::sword::core::DependencyInjectionError::ConfigInjectionError {
                        source: e,
                    })
            }
        }

        // Auto-register this config type using inventory
        // This allows automatic registration of all configs during app initialization
        // We use the full path to inventory through sword's re-export
        const _: () = {
            ::sword::__internal::inventory::submit! {
                ::sword::__internal::ConfigRegistrar::new(|config, state| {
                    <#struct_name as ::sword::core::ConfigItem>::register(config, state)
                })
            }
        };
    };

    TokenStream::from(expanded)
}
