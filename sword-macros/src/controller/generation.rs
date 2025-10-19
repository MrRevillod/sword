use proc_macro2::TokenStream;
use quote::quote;

use crate::controller::parsing::ControllerInput;
use crate::middlewares::expand_middleware_args;
use crate::shared::{generate_build_method, generate_try_from_state};

pub fn generate_controller_builder(input: &ControllerInput) -> TokenStream {
    let base_path = &input.base_path;
    let self_name = &input.struct_name;
    let self_fields = &input.fields;
    let controller_middlewares = &input.middlewares;

    let processed_middlewares: Vec<TokenStream> = controller_middlewares
        .iter()
        .map(expand_middleware_args)
        .collect();

    let try_from_impl = generate_try_from_state(self_name, self_fields);
    let build_method = generate_build_method();

    quote! {
        impl ::sword::web::ControllerBuilder for #self_name {
            fn base_path() -> &'static str {
                #base_path
            }

            fn apply_middlewares(
                router: ::sword::__internal::AxumRouter,
                state: ::sword::core::State,
            ) -> ::sword::__internal::AxumRouter {
                let mut result = router;
                #(
                    result = result.layer(#processed_middlewares);
                )*
                result
            }

            #build_method
        }

        #try_from_impl
    }
}
