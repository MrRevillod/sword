use proc_macro2::TokenStream;
use quote::quote;

use crate::{injectable::*, shared::*};

pub fn generate_component_trait(input: &InjectableInput) -> TokenStream {
    let struct_name = &input.struct_name;

    let deps_impl = gen_deps(struct_name, &input.fields);
    let build_impl = gen_build(struct_name, &input.fields);

    quote! {
        #build_impl
        #deps_impl
        impl ::sword::core::Component for #struct_name {}
    }
}

pub fn generate_provider_trait(parsed: &InjectableInput) -> TokenStream {
    let struct_name = &parsed.struct_name;

    let build_impl = quote! {
        impl ::sword::core::Build for #struct_name {
            fn build(state: &::sword::core::State) -> Result<Self, ::sword::core::DependencyInjectionError> {
                state.get::<Self>()
                    .map_err(|_| ::sword::core::DependencyInjectionError::DependencyNotFound {
                        type_name: stringify!(#struct_name).to_string(),
                    })
            }
        }
    };

    quote! {
        #build_impl

        impl ::sword::core::Provider for #struct_name {}
    }
}
