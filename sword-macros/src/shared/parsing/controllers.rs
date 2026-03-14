use crate::{controllers::web::InterceptorArgs, shared::StructFields};
use proc_macro::TokenStream;
use syn::spanned::Spanned;
use syn::{Expr, ExprLit, ExprPath, Ident, ItemStruct, Lit, Type};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ParsedControllerKind {
    Web,
    SocketIo,
    Grpc,
}

pub struct CommonControllerInput {
    pub struct_name: Ident,
    pub base_path: String,
    pub kind: ParsedControllerKind,
    pub fields: Vec<(Ident, Type)>,
    pub interceptors: Vec<InterceptorArgs>,
}

impl CommonControllerInput {
    pub fn parse(attr: TokenStream, item: TokenStream) -> syn::Result<Self> {
        let input = syn::parse::<ItemStruct>(item)?;
        let parsed_attrs = Self::parse_controller_attrs(attr, &input)?;

        let mut interceptors = Vec::new();
        let fields = StructFields::parse(&input)?;

        for attr in &input.attrs {
            if attr.path().is_ident("interceptor") {
                let args = attr.parse_args::<InterceptorArgs>()?;
                interceptors.push(args);
            }
        }

        if parsed_attrs.base_path.is_empty() {
            return Err(syn::Error::new(
                input.ident.span(),
                "Base path cannot be empty. Use \"/\" for root path",
            ));
        }

        if !parsed_attrs.base_path.starts_with('/') {
            return Err(syn::Error::new(
                input.ident.span(),
                "Controller base path must start with '/'",
            ));
        }

        Ok(Self {
            base_path: parsed_attrs.base_path,
            kind: parsed_attrs.kind,
            struct_name: input.ident,
            fields,
            interceptors,
        })
    }

    fn parse_controller_attrs(
        attr: TokenStream,
        input: &ItemStruct,
    ) -> syn::Result<ControllerAttrInput> {
        use syn::parse::Parser;

        let parser = syn::punctuated::Punctuated::<
            syn::MetaNameValue,
            syn::Token![,],
        >::parse_terminated;

        let args = parser.parse(attr)?;

        let mut kind = None;
        let mut path = None;
        let mut namespace = None;

        for arg in args {
            let Some(ident) = arg.path.get_ident().map(ToString::to_string) else {
                return Err(syn::Error::new(
                    arg.path.span(),
                    "Invalid controller attribute key",
                ));
            };

            match ident.as_str() {
                "kind" => {
                    if kind.is_some() {
                        return Err(syn::Error::new(
                            arg.path.span(),
                            "`kind` can only be specified once",
                        ));
                    }

                    kind = Some(parse_kind_expr(&arg.value)?);
                }
                "path" => {
                    if path.is_some() {
                        return Err(syn::Error::new(
                            arg.path.span(),
                            "`path` can only be specified once",
                        ));
                    }
                    path = Some(parse_string_expr(&arg.value, "path")?);
                }
                "namespace" => {
                    if namespace.is_some() {
                        return Err(syn::Error::new(
                            arg.path.span(),
                            "`namespace` can only be specified once",
                        ));
                    }
                    namespace = Some(parse_string_expr(&arg.value, "namespace")?);
                }
                _ => {
                    return Err(syn::Error::new(
                        arg.path.span(),
                        format!("Unknown controller attribute `{ident}`"),
                    ));
                }
            }
        }

        let Some(kind) = kind else {
            return Err(syn::Error::new(
                input.ident.span(),
                "Missing required `kind` in #[controller(...)]",
            ));
        };

        let base_path = match kind {
            ParsedControllerKind::Web => {
                if namespace.is_some() {
                    return Err(syn::Error::new(
                        input.ident.span(),
                        "`namespace` is not valid for Controller::Web",
                    ));
                }

                path.ok_or_else(|| {
                    syn::Error::new(
                        input.ident.span(),
                        "`path` is required for Controller::Web",
                    )
                })?
            }
            ParsedControllerKind::SocketIo => {
                if path.is_some() {
                    return Err(syn::Error::new(
                        input.ident.span(),
                        "`path` is not valid for Controller::SocketIo",
                    ));
                }

                namespace.ok_or_else(|| {
                    syn::Error::new(
                        input.ident.span(),
                        "`namespace` is required for Controller::SocketIo",
                    )
                })?
            }
            ParsedControllerKind::Grpc => {
                return Err(syn::Error::new(
                    input.ident.span(),
                    "Controller::Grpc is not implemented yet",
                ));
            }
        };

        Ok(ControllerAttrInput { kind, base_path })
    }
}

struct ControllerAttrInput {
    kind: ParsedControllerKind,
    base_path: String,
}

fn parse_kind_expr(expr: &Expr) -> syn::Result<ParsedControllerKind> {
    let Expr::Path(ExprPath { path, .. }) = expr else {
        return Err(syn::Error::new(
            expr.span(),
            "`kind` must be a path like `Controller::Web`",
        ));
    };

    let Some(last) = path
        .segments
        .last()
        .map(|segment| segment.ident.to_string())
    else {
        return Err(syn::Error::new(expr.span(), "Invalid `kind` value"));
    };

    match last.as_str() {
        "Web" => Ok(ParsedControllerKind::Web),
        "SocketIo" => Ok(ParsedControllerKind::SocketIo),
        "Grpc" => Ok(ParsedControllerKind::Grpc),
        _ => Err(syn::Error::new(
            expr.span(),
            "`kind` must be one of Controller::Web, Controller::SocketIo, Controller::Grpc",
        )),
    }
}

fn parse_string_expr(expr: &Expr, field: &str) -> syn::Result<String> {
    let Expr::Lit(ExprLit {
        lit: Lit::Str(lit_str),
        ..
    }) = expr
    else {
        return Err(syn::Error::new(
            expr.span(),
            format!("`{field}` must be a string literal"),
        ));
    };

    Ok(lit_str.value())
}
