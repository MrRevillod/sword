use proc_macro2::TokenStream;
use quote::quote;

use crate::{
    adapters::rest::interceptor::expand_interceptor_args,
    shared::{CommonHttpAdapterInput, gen_build, gen_clone, gen_deps},
};

pub fn generate_controller_builder(input: &CommonHttpAdapterInput) -> TokenStream {
    let base_path = &input.base_path;
    let self_name = &input.struct_name;
    let self_fields = &input.fields;
    let controller_interceptors = &input.interceptors;

    let processed_interceptors: Vec<TokenStream> = controller_interceptors
        .iter()
        .map(expand_interceptor_args)
        .collect();

    let deps_impl = gen_deps(self_name, self_fields);
    let build_impl = gen_build(self_name, self_fields);
    let clone_impl = gen_clone(self_name, self_fields);

    quote! {
        #build_impl
        #deps_impl
        #clone_impl

        impl ::sword::adapters::rest::RestAdapter for #self_name {
            fn base_path() -> &'static str {
                #base_path
            }

            fn apply_middlewares(
                router: ::sword::internal::axum::AxumRouter,
                state: ::sword::internal::core::State,
            ) -> ::sword::internal::axum::AxumRouter {
                let mut result = router;
                #(
                    result = result.layer(#processed_interceptors);
                )*
                result
            }
        }
    }
}
