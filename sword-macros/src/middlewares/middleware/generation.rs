use proc_macro2::TokenStream;
use quote::quote;

use crate::{middlewares::MiddlewareInput, shared::*};

pub fn generate_middleware_builder(input: &MiddlewareInput) -> TokenStream {
    let self_name = &input.struct_name;
    let self_fields = &input.fields;

    let error_type = quote! { ::sword::core::DependencyInjectionError };

    let build_impl = gen_build(self_name, self_fields, &error_type);
    let clone_impl = gen_clone(self_name, self_fields);

    quote! {
        #build_impl
        #clone_impl

        impl ::sword::core::Clonable for #self_name {}

        impl ::sword::web::Middleware for #self_name {}

        ::sword::__internal::inventory::submit! {
            ::sword::web::MiddlewareRegistrar::new::<#self_name>()
        }
    }
}
