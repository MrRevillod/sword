use quote::ToTokens;
use std::str::FromStr;
use syn::{
    Error, FnArg, Ident, ImplItem, ImplItemFn, ItemImpl, LitStr, Pat, PatIdent,
    PatType, Path, Type, parse as syn_parse, spanned::Spanned,
};

#[derive(Default)]
pub enum EventKind {
    OnConnection(Vec<Path>),
    OnDisconnection,
    OnMessage(String),

    #[default]
    Fallback,
}

impl EventKind {
    pub fn is_on_connection(&self) -> bool {
        matches!(self, EventKind::OnConnection(_))
    }

    pub fn is_on_message(&self) -> bool {
        matches!(self, EventKind::OnMessage(_))
    }
}

pub struct HandlerInfo {
    pub name: Ident,
    pub event_kind: EventKind,
    pub args: Vec<(Ident, Type)>,
}

pub fn parse_handlers(input: &ItemImpl) -> syn::Result<Vec<HandlerInfo>> {
    let mut handlers: Vec<HandlerInfo> = vec![];

    for item in &input.items {
        if !matches!(item, ImplItem::Fn(_)) {
            continue;
        }

        let Ok(handler) = syn_parse::<ImplItemFn>(item.to_token_stream().into())
        else {
            return Err(Error::new(item.span(), "Failed to parse handler function"));
        };

        let mut event_kind = EventKind::default();
        let mut interceptors = Vec::new();

        for attr in &handler.attrs {
            let Some(ident) = attr.path().get_ident() else {
                continue;
            };

            if *ident == "interceptor" {
                interceptors.push(attr.parse_args::<Path>()?);
                continue;
            }

            event_kind = match EventKind::from_str(&ident.to_string()) {
                Ok(kind) => kind,
                Err(error) => return Err(Error::new(item.span(), error)),
            };

            if event_kind.is_on_message() {
                let args = attr.parse_args::<LitStr>()?;
                event_kind = EventKind::OnMessage(args.value());
            }

            if event_kind.is_on_connection()
                && attr.meta.require_path_only().is_err()
            {
                event_kind = EventKind::OnConnection(interceptors.clone());
            }
        }

        if event_kind.is_on_connection() && !interceptors.is_empty() {
            event_kind = EventKind::OnConnection(interceptors);
        }

        handlers.push(HandlerInfo {
            name: handler.sig.ident.clone(),
            event_kind,
            args: extract_function_args(&handler),
        });
    }

    Ok(handlers)
}

impl FromStr for EventKind {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let variant = match s {
            "on_connection" => Self::OnConnection(Vec::new()),
            "on_disconnection" => Self::OnDisconnection,
            "on_fallback" => Self::Fallback,
            event if event.starts_with("on_message") => {
                Self::OnMessage(String::default())
            }
            _ => {
                return Err(format!(
                    "Invalid event kind '{}'. Expected 'on_connection', 'on_disconnection', 'on_fallback', or 'on_message'.",
                    s
                ));
            }
        };

        Ok(variant)
    }
}

fn extract_function_args(func: &ImplItemFn) -> Vec<(Ident, Type)> {
    func.sig
        .inputs
        .iter()
        .filter_map(|arg| match arg {
            FnArg::Receiver(_) => None,
            FnArg::Typed(PatType { pat, ty, .. }) => {
                let ident = extract_pat_ident(pat)?;
                Some((ident, (**ty).clone()))
            }
        })
        .collect()
}

fn extract_pat_ident(pat: &Pat) -> Option<Ident> {
    match pat {
        Pat::Ident(PatIdent { ident, .. }) => Some(ident.clone()),
        Pat::TupleStruct(tuple_struct) => tuple_struct.path.get_ident().cloned(),
        _ => None,
    }
}
