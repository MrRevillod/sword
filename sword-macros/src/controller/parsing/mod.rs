mod attributes;

use proc_macro::TokenStream;
use syn::{Ident, ItemStruct, Type};

use crate::{
    controller::parsing::attributes::ControllerArgs, middlewares::MiddlewareArgs,
    shared::collect_struct_fields,
};

pub struct ControllerInput {
    pub struct_name: Ident,
    pub base_path: String,
    pub fields: Vec<(Ident, Type)>,
    pub middlewares: Vec<MiddlewareArgs>,
}

pub fn parse_controller_input(
    attr: TokenStream,
    item: TokenStream,
) -> Result<ControllerInput, syn::Error> {
    let input = syn::parse::<ItemStruct>(item)?;
    let args = syn::parse::<ControllerArgs>(attr)?;

    let mut middlewares = vec![];
    let fields = collect_struct_fields(&input);

    for attr in &input.attrs {
        if attr.path().is_ident("middleware") {
            let args = attr.parse_args::<MiddlewareArgs>()?;
            middlewares.push(args);
        }
    }

    if args.base_path.is_empty() {
        return Err(syn::Error::new(
            input.ident.span(),
            "Base path cannot be empty. Use \"/\" for root path",
        ));
    }

    if !args.base_path.starts_with('/') {
        return Err(syn::Error::new(
            input.ident.span(),
            "Controller base path must start with '/'",
        ));
    }

    let base_path = match args.version {
        Some(ver) => format!("/{}/{}", ver, args.base_path.trim_start_matches('/')),
        None => args.base_path,
    };

    Ok(ControllerInput {
        base_path,
        struct_name: input.ident,
        fields,
        middlewares,
    })
}
