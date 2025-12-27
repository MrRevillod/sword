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
        impl ::sword::internal::core::Component for #struct_name {}
    }
}

pub fn generate_provider_trait(parsed: &InjectableInput) -> TokenStream {
    let struct_name = &parsed.struct_name;

    quote! {
        impl ::sword::internal::core::Provider for #struct_name {}
    }
}
