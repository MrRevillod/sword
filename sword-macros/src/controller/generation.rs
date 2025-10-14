use proc_macro2::TokenStream;
use quote::quote;
use syn::{Ident, Type};

use crate::controller::parsing::ControllerInput;
use crate::middleware::expand_middleware_args;
use crate::shared::generate_field_extractions;

pub fn generate_controller_builder(input: &ControllerInput) -> TokenStream {
    let base_path = &input.base_path;
    let self_name = &input.struct_name;
    let self_fields = &input.fields;
    let controller_middlewares = &input.middlewares;

    let field_extractions = generate_field_extractions(self_fields);
    let field_assignments = generate_field_assignments(self_fields);

    let processed_middlewares: Vec<TokenStream> = controller_middlewares
        .iter()
        .map(expand_middleware_args)
        .collect();

    quote! {

        impl ::sword::web::ControllerBuilder for #self_name {

            fn base_path() -> &'static str {
                #base_path
            }

            fn apply_controller_middlewares(
                router: ::sword::__internal::AxumRouter,
                state: ::sword::core::State,
            ) -> ::sword::__internal::AxumRouter {
                let mut result = router;

                #(
                    result = result.layer(#processed_middlewares);
                )*

                result
            }

            fn build(state: ::sword::core::State) -> Result<Self, ::sword::errors::DependencyInjectionError> {
                Self::try_from(&state)
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
    }
}

fn generate_field_assignments(fields: &[(Ident, Type)]) -> TokenStream {
    let assignments = fields.iter().map(|(name, _)| {
        quote! { #name }
    });

    quote! {
        #(#assignments),*
    }
}
