//! Parsing logic for WebSocket macros

use syn::{
    Attribute, LitStr, Result as SynResult,
    parse::{Parse, ParseStream},
};

pub struct WebSocketPath {
    pub path: String,
}

impl Parse for WebSocketPath {
    fn parse(input: ParseStream) -> SynResult<Self> {
        let lit_str: LitStr = input.parse()?;
        Ok(WebSocketPath {
            path: lit_str.value(),
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HandlerType {
    OnConnection,
    OnDisconnect,
    SubscribeMessage,
    OnFallback,
}

pub fn get_handler_type(
    attrs: &[Attribute],
) -> Option<(HandlerType, Option<String>)> {
    for attr in attrs {
        if attr.path().is_ident("on_connection") {
            return Some((HandlerType::OnConnection, None));
        }
        if attr.path().is_ident("on_disconnect") {
            return Some((HandlerType::OnDisconnect, None));
        }
        if attr.path().is_ident("on_fallback") {
            return Some((HandlerType::OnFallback, None));
        }
        if attr.path().is_ident("subscribe_message") {
            if let Ok(message_type) = attr.parse_args::<LitStr>() {
                return Some((
                    HandlerType::SubscribeMessage,
                    Some(message_type.value()),
                ));
            }
        }
    }
    None
}
