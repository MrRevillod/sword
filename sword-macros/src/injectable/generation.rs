use proc_macro2::TokenStream;
use quote::quote;
use syn::Ident;

use crate::{injectable::*, shared::*};

pub fn generate_injectable_component_trait(
    input: &InjectableStructInput,
) -> TokenStream {
    let field_extractions = generate_field_extraction_from_state(&input.fields);
    let field_assignments = generate_struct_field_assignments(&input.fields);

    let struct_name = &input.struct_name;

    let type_ids = input.fields.iter().map(|(_, ty)| {
        quote! {
            std::any::TypeId::of::<#ty>()
        }
    });

    quote! {
        impl ::sword::core::Component for #struct_name {
            fn build(state: &::sword::core::State) -> Result<Self, ::sword::errors::DependencyInjectionError> {
                Self::try_from(state)
            }

            fn dependencies() -> Vec<std::any::TypeId> {
                vec![
                    #(#type_ids),*
                ]
            }
        }

        impl TryFrom<&::sword::core::State> for #struct_name {
            type Error = ::sword::errors::DependencyInjectionError;

            fn try_from(state: &::sword::core::State) -> Result<Self, Self::Error> {
                #field_extractions

                Ok(Self {
                    #field_assignments
                })
            }
        }
    }
}

pub fn generate_injectable_provider_trait(
    parsed: &InjectableStructInput,
) -> TokenStream {
    let struct_name = &parsed.struct_name;

    quote! {
        impl ::sword::core::Provider for #struct_name {}

        impl TryFrom<&::sword::core::State> for #struct_name {
            type Error = ::sword::errors::DependencyInjectionError;

            fn try_from(state: &::sword::core::State) -> Result<Self, Self::Error> {
                state.get::<Self>()
                    .map_err(|_| ::sword::errors::DependencyInjectionError::DependencyNotFound {
                        type_name: stringify!(#struct_name).to_string(),
                    })
            }
        }
    }
}

pub fn generate_clone_impl(parsed: &InjectableStructInput) -> TokenStream {
    let struct_name = &parsed.struct_name;
    let field_names: Vec<&Ident> =
        parsed.fields.iter().map(|(name, _)| name).collect();

    quote! {
        impl ::std::clone::Clone for #struct_name {
            fn clone(&self) -> Self {
                Self {
                    #(#field_names: self.#field_names.clone()),*
                }
            }
        }
    }
}
