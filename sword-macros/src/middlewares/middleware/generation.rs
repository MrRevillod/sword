use proc_macro2::TokenStream;
use quote::quote;
use syn::Ident;

use crate::{
    middlewares::MiddlewareInput,
    shared::{generate_field_assignments, generate_field_extractions},
};

pub fn generate_middleware_builder(input: &MiddlewareInput) -> TokenStream {
    let self_name = &input.struct_name;
    let self_fields = &input.fields;

    let field_extractions = generate_field_extractions(self_fields);
    let field_assignments = generate_field_assignments(self_fields);

    let field_names: Vec<&Ident> =
        input.fields.iter().map(|(name, _)| name).collect();

    quote! {

        impl ::sword::web::Middleware for #self_name {
            fn build(state: &::sword::core::State) -> Result<Self, ::sword::errors::DependencyInjectionError> {
                Self::try_from(state)
            }
        }

        impl TryFrom<&::sword::core::State> for #self_name {
            type Error = ::sword::errors::DependencyInjectionError;

            fn try_from(state: &::sword::core::State) -> Result<Self, Self::Error> {
                #field_extractions

                Ok(Self {
                    #field_assignments
                })
            }
        }

        ::sword::__internal::inventory::submit! {
            ::sword::web::MiddlewareRegistrar::new::<#self_name>()
        }


        impl ::std::clone::Clone for #self_name {
            fn clone(&self) -> Self {
                Self {
                    #(#field_names: self.#field_names.clone()),*
                }
            }
        }
    }
}
