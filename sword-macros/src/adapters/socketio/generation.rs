use crate::shared::{CommonHttpAdapterInput, gen_build, gen_clone, gen_deps};

use proc_macro2::TokenStream;
use quote::quote;

pub fn generate_socketio_adapter_builder(
    input: &CommonHttpAdapterInput,
) -> TokenStream {
    let path = &input.base_path;
    let self_name = &input.struct_name;
    let self_fields = &input.fields;

    let deps_impl = gen_deps(self_name, self_fields);
    let build_impl = gen_build(self_name, self_fields);
    let clone_impl = gen_clone(self_name, self_fields);

    quote! {
        #build_impl
        #deps_impl
        #clone_impl

        impl ::sword::internal::core::SocketIoAdapter for #self_name {
            fn namespace() -> &'static str {
                #path
            }
        }
    }
}
