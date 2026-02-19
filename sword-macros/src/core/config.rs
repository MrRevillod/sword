use proc_macro::TokenStream;
use quote::quote;
use syn::{DeriveInput, Error, Expr, Lit, Meta};

pub fn expand_config_struct(
    args: TokenStream,
    input: &DeriveInput,
) -> syn::Result<TokenStream> {
    let struct_name = &input.ident;
    let self_ty = quote! { #struct_name };
    let meta = syn::parse::<Meta>(args)?;

    let nv = meta.require_name_value().map_err(|_| {
        Error::new_spanned(
            &meta,
            r#"expected format: #[config(key = "section_name")]"#,
        )
    })?;

    if !nv.path.is_ident("key") {
        return Err(Error::new_spanned(&nv.path, "expected `key` attribute"));
    }

    let Expr::Lit(expr_lit) = &nv.value else {
        return Err(Error::new_spanned(
            &nv.value,
            "expected string literal for key",
        ));
    };

    let Lit::Str(lit_str) = &expr_lit.lit else {
        return Err(Error::new_spanned(
            &expr_lit.lit,
            "expected string literal for key",
        ));
    };

    let expanded = quote! {
        #input

        impl ::sword::internal::core::ConfigItem for #struct_name {
            fn key() -> &'static str {
                #lit_str
            }
        }

        impl TryFrom<&::sword::internal::core::State> for #struct_name {
            type Error = ::sword::internal::core::DependencyInjectionError;

            fn try_from(state: &::sword::internal::core::State) -> Result<Self, Self::Error> {
                let config = state.get::<::sword::internal::core::Config>()
                    .map_err(|_| ::sword::internal::core::DependencyInjectionError::DependencyNotFound {
                        type_name: "Config".to_string(),
                    })?;

                config.get::<Self>()
                    .ok_or_else(|| ::sword::internal::core::DependencyInjectionError::DependencyNotFound {
                        type_name: format!("Config item '{}'", Self::key()),
                    })
            }
        }

        // Auto-register this config type using inventory
        // This allows automatic registration of all configs during app initialization
        // We use the full path to inventory through sword's re-export
        const _: () = {
            ::sword::internal::inventory::submit! {
                ::sword::internal::core::ConfigRegistrar::new(|state, config| {
                    let config_item = config.get::<#self_ty>().unwrap_or_else(|| {
                          panic!(
                              "Failed to load config item '{}' from the configuration file",
                              #lit_str
                          )
                    });

                    state.insert(config_item);
                })
            }
        };
    };

    Ok(TokenStream::from(expanded))
}
