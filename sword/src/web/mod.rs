mod controller;
mod middlewares;
mod request;
mod response;

#[cfg(feature = "websocket")]
pub mod websocket;

pub use axum::http::{Method, StatusCode, header as headers};

pub use controller::*;
pub use middlewares::{
    Middleware, MiddlewareResult, MiddlewaresConfig, Next, OnRequest,
    OnRequestWithConfig, middleware, uses,
};

pub use sword_layers::helmet;

pub use request::cookies;
pub use request::{Request, RequestError};
pub use response::*;
pub use websocket::*;

#[cfg(feature = "multipart")]
pub use request::multipart;

#[cfg(feature = "validator")]
pub use request::validator as request_validator;

#[doc(hidden)]
pub mod __internal {
    pub use super::middlewares::__internal::MiddlewareRegistrar;
}
