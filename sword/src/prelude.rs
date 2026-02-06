pub use crate::adapters::AdapterRegistry;
pub use crate::application::*;
pub use crate::module::Module;
pub use sword_macros::{Interceptor, interceptor, main};

pub use sword_core::{
    ComponentRegistry, Config, Provider, ProviderRegistry, config, injectable,
};

#[cfg(feature = "validation-validator")]
pub use validator::Validate;

#[cfg(feature = "adapter-http-controllers")]
pub use crate::adapters::http::{
    ContentDisposition, File, FromRequest, FromRequestParts, HttpError,
    HttpInterceptorResult, HttpResult, JsonResponse, JsonResponseBody, Next,
    OnRequest, OnRequestWithConfig, Redirect, Request, RequestError, controller,
    cookies as sword_cookies, delete, get, patch, post, put,
};

pub use axum::body::Bytes;
pub use axum::http::{HeaderMap as Headers, Method, Uri};

#[cfg(all(
    feature = "validation-validator",
    feature = "adapter-http-controllers"
))]
pub use crate::adapters::http::ValidatorRequestValidation;

#[cfg(all(feature = "multipart", feature = "adapter-http-controllers"))]
pub use crate::adapters::http::multipart as sword_multipart;

#[cfg(feature = "adapter-socketio")]
pub use crate::adapters::socketio::{
    AckSender, Data, DisconnectReason, Event, Extension, HttpExtension,
    LocalAdapter, MaybeExtension, MaybeHttpExtension, OnConnect, ProtocolVersion,
    SocketContext, SocketError, SocketIo, SocketRef, TransportType, TryData, on,
    socketio_adapter,
};

#[cfg(feature = "adapter-socketio")]
pub use crate::runtimes::http::SocketIoParser;

#[doc(hidden)]
pub use sword_core::{
    Build, Component, ConfigItem, FromState, FromStateArc, HasDeps,
};

#[doc(hidden)]
pub use crate::{adapters::Adapter, interceptor::Interceptor};

#[doc(hidden)]
#[cfg(feature = "adapter-http-controllers")]
pub use crate::adapters::http::HttpController;

#[doc(hidden)]
#[cfg(feature = "adapter-socketio")]
pub use crate::adapters::socketio::SocketIoAdapter;
