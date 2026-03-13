pub use crate::adapters::AdapterRegistry;
pub use crate::application::*;
pub use crate::module::Module;
pub use sword_macros::{Interceptor, interceptor, main};

pub use sword_core::{
    ComponentRegistry, Config, Provider, ProviderRegistry, config, injectable,
};

#[cfg(feature = "validation-validator")]
pub use validator::Validate;

#[cfg(feature = "web-adapter-controllers")]
pub use crate::adapters::controllers::{
    ContentDisposition, File, FromRequest, FromRequestParts, HttpError,
    HttpInterceptorResult, JsonResponse, JsonResponseBody, Next, OnRequest,
    OnRequestStream, OnRequestStreamWithConfig, OnRequestWithConfig, Redirect,
    Request, RequestError, Result, StreamRequest, controller,
    cookies as sword_cookies, delete, get, patch, post, put,
};

pub use axum::body::Bytes;
pub use axum::http::{HeaderMap as Headers, Method, Uri};

#[cfg(all(
    feature = "validation-validator",
    feature = "web-adapter-controllers"
))]
pub use crate::adapters::controllers::ValidatorRequestValidation;

#[cfg(all(feature = "multipart", feature = "web-adapter-controllers"))]
pub use crate::adapters::controllers::multipart as sword_multipart;

#[cfg(feature = "web-adapter-socketio")]
pub use crate::adapters::socketio::{
    AckSender, Data, DisconnectReason, Event, Extension, HttpExtension,
    LocalAdapter, MaybeExtension, MaybeHttpExtension, OnConnect, ProtocolVersion,
    SocketContext, SocketError, SocketIo, SocketRef, TransportType, TryData, on,
    socketio_adapter,
};

#[cfg(feature = "web-adapter-socketio")]
pub use crate::runtimes::web::SocketIoParser;

#[doc(hidden)]
pub use sword_core::{
    Build, Component, ConfigItem, FromState, FromStateArc, HasDeps,
};

#[doc(hidden)]
pub use crate::{adapters::Adapter, interceptor::Interceptor};

#[doc(hidden)]
#[cfg(feature = "web-adapter-controllers")]
pub use crate::adapters::controllers::HttpController;

#[doc(hidden)]
#[cfg(feature = "web-adapter-socketio")]
pub use crate::adapters::socketio::SocketIoAdapter;
