pub use crate::adapters::AdapterRegistry;
pub use crate::application::*;
pub use crate::module::Module;
pub use sword_macros::{Interceptor, interceptor, main};

pub use sword_core::{
    ComponentRegistry, Config, Provider, ProviderRegistry, config, injectable,
};

#[cfg(feature = "validation-validator")]
pub use validator::Validate;

pub use crate::adapters::rest::{
    ContentDisposition, File, FromRequest, FromRequestParts, HttpError,
    HttpInterceptorResult, HttpResult, JsonResponse, JsonResponseBody, Next,
    OnRequest, OnRequestWithConfig, Redirect, Request, RequestError, controller,
    cookies as sword_cookies, delete, get, patch, post, put, rest_adapter,
};

pub use crate::adapters::rest::extractors::{
    Extension as RestExtension, Header, Json, PathParams, Query,
};

pub use axum::body::Bytes;
pub use axum::http::{HeaderMap as Headers, Method, Uri};

#[cfg(feature = "validation-validator")]
pub use crate::adapters::rest::{
    ValidatorRequestValidation,
    extractors::{ValidatedJson, ValidatedPath, ValidatedQuery},
};

#[cfg(feature = "multipart")]
pub use crate::adapters::rest::multipart as sword_multipart;

#[cfg(feature = "adapter-socketio")]
pub use crate::adapters::socketio::{
    AckSender, Data, DisconnectReason, Event, Extension, HttpExtension,
    LocalAdapter, MaybeExtension, MaybeHttpExtension, OnConnect, ProtocolVersion,
    SocketContext, SocketError, SocketIo, SocketRef, TransportType, TryData, on,
    socketio_adapter,
};

#[doc(hidden)]
pub use sword_core::{
    Build, Component, ConfigItem, FromState, FromStateArc, HasDeps,
};

#[doc(hidden)]
pub use crate::{
    adapters::{Adapter, rest::RestAdapter},
    interceptor::Interceptor,
};

#[doc(hidden)]
#[cfg(feature = "adapter-socketio")]
pub use crate::adapters::socketio::SocketIoAdapter;
