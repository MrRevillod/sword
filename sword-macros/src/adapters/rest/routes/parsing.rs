use crate::{adapters::rest::interceptor::InterceptorArgs, shared::PATH_KIND_REGEX};

use proc_macro2::Ident;
use quote::ToTokens;
use syn::{
    Attribute, Error, ImplItem, ImplItemFn, ItemImpl, LitStr, parse as syn_parse,
    spanned::Spanned,
};

const VALID_ROUTE_MACROS: &[&str; 7] = &[
    "get",
    "post",
    "put",
    "patch",
    "delete",
    "middleware",
    "uses",
];

pub const HTTP_METHODS: [&str; 5] = ["get", "post", "put", "delete", "patch"];

pub struct RouteInfo {
    pub method: String,
    pub path: String,
    pub handler_name: Ident,
    pub interceptors: Vec<InterceptorArgs>,
    pub needs_context: bool,
}

pub fn parse_routes(input: &ItemImpl) -> syn::Result<Vec<RouteInfo>> {
    let mut routes: Vec<RouteInfo> = vec![];

    for item in &input.items {
        if !matches!(item, ImplItem::Fn(_)) {
            continue;
        }

        let Ok(handler) = syn_parse::<ImplItemFn>(item.to_token_stream().into())
        else {
            return Err(Error::new(item.span(), "Failed to parse handler function"));
        };

        let mut route_path = String::new();
        let mut route_method = String::new();
        let mut interceptors: Vec<InterceptorArgs> = vec![];

        for attr in &handler.attrs {
            let Some(ident) = attr.path().get_ident() else {
                continue;
            };

            let ident_str = ident.to_string();

            if !VALID_ROUTE_MACROS.contains(&ident_str.as_str()) {
                continue;
            }

            if ident == "uses" {
                let args = attr.parse_args::<InterceptorArgs>()?;
                interceptors.push(args);
            } else if HTTP_METHODS.contains(&ident_str.as_str()) {
                route_method = ident.to_string();
                route_path = parse_route_path(attr)?.value();
            }
        }

        let needs_context = handler
            .sig
            .inputs
            .iter()
            .any(|arg| matches!(arg, syn::FnArg::Typed(_)));

        routes.push(RouteInfo {
            method: route_method,
            path: route_path,
            handler_name: handler.sig.ident.clone(),
            interceptors,
            needs_context,
        });
    }

    Ok(routes)
}

pub fn parse_route_path(attr: &Attribute) -> syn::Result<LitStr> {
    let Ok(path) = attr.parse_args::<LitStr>() else {
        return Err(Error::new(
            attr.span(),
            "Expected a string literal as path in HTTP method attribute, e.g., #[get(\"/path\")]",
        ));
    };

    let value = path.value();

    if !PATH_KIND_REGEX.is_match(&value) {
        return Err(Error::new(
            path.span(),
            "Invalid path format. Paths must start with '/' and can include:\n\
             - Static segments: /users\n\
             - Dynamic segments: /users/{id}\n\
             - Wildcard segments: /files/{*path}\n\
            ",
        ));
    }

    Ok(path)
}
