use proc_macro2::TokenStream;
use quote::quote;

use crate::{middlewares::MiddlewareInput, shared::*};

pub fn generate_middleware_builder(input: &MiddlewareInput) -> TokenStream {
    let self_name = &input.struct_name;
    let self_fields = &input.fields;

    let try_from_impl = generate_try_from_state(self_name, self_fields);
    let clone_impl = generate_clone_impl(self_name, self_fields);
    let build_method = generate_build_method();

    quote! {
        impl ::sword::web::Middleware for #self_name {
            #build_method
        }

        #try_from_impl

        #clone_impl

        ::sword::__internal::inventory::submit! {
            ::sword::web::MiddlewareRegistrar::new::<#self_name>()
        }
    }
}
