use proc_macro2::TokenStream;
use quote::quote;
use syn::{Ident, Type};

use super::generation::{generate_field_assignments, generate_field_extractions};

pub fn generate_try_from_state(
    struct_name: &Ident,
    fields: &[(Ident, Type)],
) -> TokenStream {
    let field_extractions = generate_field_extractions(fields);
    let field_assignments = generate_field_assignments(fields);

    quote! {
        impl TryFrom<&::sword::core::State> for #struct_name {
            type Error = ::sword::core::DependencyInjectionError;

            fn try_from(state: &::sword::core::State) -> Result<Self, Self::Error> {
                #field_extractions

                Ok(Self {
                    #field_assignments
                })
            }
        }
    }
}

pub fn generate_clone_impl(
    struct_name: &Ident,
    fields: &[(Ident, Type)],
) -> TokenStream {
    let field_names: Vec<&Ident> = fields.iter().map(|(name, _)| name).collect();

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

pub fn generate_build_method() -> TokenStream {
    quote! {
        fn build(state: &::sword::core::State) -> Result<Self, ::sword::core::DependencyInjectionError> {
            Self::try_from(state)
        }
    }
}
