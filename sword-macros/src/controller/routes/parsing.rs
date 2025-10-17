use proc_macro2::Ident;
use quote::ToTokens;
use regex_lite::Regex;
use std::sync::LazyLock;
use syn::{
    Attribute, Error, ImplItem, ImplItemFn, ItemImpl, LitStr, parse as syn_parse,
    spanned::Spanned,
};

use crate::middlewares::MiddlewareArgs;

const VALID_ROUTE_MACROS: &[&str; 6] =
    &["get", "post", "put", "patch", "delete", "middleware"];

pub const HTTP_METHODS: [&str; 5] = ["get", "post", "put", "delete", "patch"];

static PATH_KIND_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^\/(?:[^\/{}:]+|\{[^*{}][^{}]*\}|\{\*[^{}]+\})*(?:\/(?:[^\/{}:]+|\{[^*{}][^{}]*\}|\{\*[^{}]+\}))*$").unwrap()
});

pub struct RouteInfo {
    pub method: String,
    pub path: String,
    pub handler_name: Ident,
    pub middlewares: Vec<MiddlewareArgs>,
    pub needs_context: bool,
}

pub fn parse_routes(input: &ItemImpl) -> Result<Vec<RouteInfo>, syn::Error> {
    let mut routes: Vec<RouteInfo> = vec![];

    for item in input.items.iter() {
        if !matches!(item, ImplItem::Fn(_)) {
            continue;
        }

        let Ok(handler) = syn_parse::<ImplItemFn>(item.to_token_stream().into())
        else {
            return Err(Error::new(item.span(), "Failed to parse handler function"));
        };

        let mut route_path = String::new();
        let mut route_method = String::new();
        let mut middlewares: Vec<MiddlewareArgs> = vec![];

        for attr in &handler.attrs {
            let Some(ident) = attr.path().get_ident() else {
                continue;
            };

            if !VALID_ROUTE_MACROS.contains(&ident.to_string().as_str()) {
                continue;
            }

            if ident == "use_middleware" {
                let args = attr.parse_args::<MiddlewareArgs>()?;
                middlewares.push(args);
            } else if HTTP_METHODS.contains(&ident.to_string().as_str()) {
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
            middlewares,
            needs_context,
        });
    }

    Ok(routes)
}

pub fn parse_route_path(attr: &Attribute) -> Result<LitStr, syn::Error> {
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
