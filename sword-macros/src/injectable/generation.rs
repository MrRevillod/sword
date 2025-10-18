use proc_macro2::TokenStream;
use quote::quote;

use crate::{injectable::*, shared::*};

pub fn generate_component_trait(input: &InjectableInput) -> TokenStream {
    let struct_name = &input.struct_name;

    let try_from_impl = generate_try_from_state(struct_name, &input.fields);

    let type_ids = input.fields.iter().map(|(_, ty)| {
        if let Some(inner_type) = extract_arc_inner_type(ty) {
            quote! { std::any::TypeId::of::<#inner_type>() }
        } else {
            quote! { std::any::TypeId::of::<#ty>() }
        }
    });

    quote! {
        impl ::sword::core::Component for #struct_name {
            fn build(state: &::sword::core::State) -> Result<Self, ::sword::errors::DependencyInjectionError> {
                Self::try_from(state)
            }

            fn dependencies() -> Vec<std::any::TypeId> {
                vec![#(#type_ids),*]
            }
        }

        #try_from_impl
    }
}

pub fn generate_provider_trait(parsed: &InjectableInput) -> TokenStream {
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
