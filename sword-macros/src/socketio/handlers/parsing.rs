use quote::ToTokens;
use std::str::FromStr;
use syn::{
    Error, FnArg, Ident, ImplItem, ImplItemFn, ItemImpl, LitStr, Pat, PatIdent,
    PatType, Type, parse as syn_parse, spanned::Spanned,
};

#[derive(Default, PartialEq, Eq)]
pub enum EventKind {
    #[default]
    OnConnection,
    OnDisconnection,
    OnMessage(String),
    Fallback,
}

#[derive(PartialEq)]
pub struct HandlerInfo {
    pub name: Ident,
    pub event_kind: EventKind,
    pub args: Vec<(Ident, Type)>,
}

pub struct CategorizedHandlers<'a> {
    pub on_connection: Option<&'a HandlerInfo>,
    pub on_disconnection: Option<&'a HandlerInfo>,
    pub on_fallback: Option<&'a HandlerInfo>,
    pub message_handlers: Vec<(&'a String, &'a HandlerInfo)>,
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

        for attr in &handler.attrs {
            let Some(ident) = attr.path().get_ident() else {
                continue;
            };

            event_kind = match EventKind::from_str(&ident.to_string()) {
                Ok(kind) => kind,
                Err(error) => return Err(Error::new(item.span(), error)),
            };

            if event_kind == EventKind::OnMessage(String::default()) {
                let args = attr.parse_args::<LitStr>()?;
                event_kind = EventKind::OnMessage(args.value());
            }
        }

        handlers.push(HandlerInfo {
            name: handler.sig.ident.clone(),
            event_kind,
            args: extract_function_args(&handler),
        });
    }

    Ok(handlers)
}

pub fn categorize<'a>(handlers: &'a [HandlerInfo]) -> CategorizedHandlers<'a> {
    let mut on_connection = None;
    let mut on_disconnection = None;
    let mut on_fallback = None;
    let mut message_handlers = Vec::new();

    for handler in handlers {
        match &handler.event_kind {
            EventKind::OnConnection => on_connection = Some(handler),
            EventKind::OnDisconnection => on_disconnection = Some(handler),
            EventKind::Fallback => on_fallback = Some(handler),
            EventKind::OnMessage(event_name) => {
                message_handlers.push((event_name, handler));
            }
        }
    }

    CategorizedHandlers {
        on_connection,
        on_disconnection,
        on_fallback,
        message_handlers,
    }
}

impl FromStr for EventKind {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let variant = match s {
            "on_connection" => Self::OnConnection,
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
