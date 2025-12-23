use crate::{middlewares::MiddlewareArgs, shared::StructFields};
use proc_macro::TokenStream;
use syn::{Ident, ItemStruct, LitStr, Type};

pub struct CommonHttpAdapterInput {
    pub struct_name: Ident,
    pub base_path: String,
    pub fields: Vec<(Ident, Type)>,
    pub middlewares: Vec<MiddlewareArgs>,
}

impl CommonHttpAdapterInput {
    pub fn parse(attr: TokenStream, item: TokenStream) -> syn::Result<Self> {
        let input = syn::parse::<ItemStruct>(item)?;
        let base_path = syn::parse::<LitStr>(attr)?.value();

        let mut middlewares = Vec::new();
        let fields = StructFields::parse(&input)?;

        for attr in &input.attrs {
            if attr.path().is_ident("uses") {
                middlewares.push(attr.parse_args::<MiddlewareArgs>()?);
            }
        }

        if base_path.is_empty() {
            return Err(syn::Error::new(
                input.ident.span(),
                "Base path cannot be empty. Use \"/\" for root path",
            ));
        }

        if !base_path.starts_with('/') {
            return Err(syn::Error::new(
                input.ident.span(),
                "Controller base path must start with '/'",
            ));
        }

        Ok(CommonHttpAdapterInput {
            base_path,
            struct_name: input.ident,
            fields,
            middlewares,
        })
    }
}
