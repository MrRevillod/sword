use proc_macro2::TokenStream;
use quote::quote;

use crate::{middlewares::MiddlewareInput, shared::*};

pub fn generate_middleware_builder(input: &MiddlewareInput) -> TokenStream {
    let self_name = &input.struct_name;
    let self_fields = &input.fields;

    let build_impl = gen_build(self_name, self_fields);
    let clone_impl = gen_clone(self_name, self_fields);

    quote! {
        #build_impl
        #clone_impl

        impl ::sword::web::Middleware for #self_name {}

        ::sword::__internal::inventory::submit! {
            ::sword::web::middleware_registrar::MiddlewareRegistrar::new::<#self_name>()
        }
    }
}
