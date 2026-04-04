#![allow(clippy::new_ret_no_self)]

pub mod layer_stack;

#[cfg(feature = "body-limit")]
pub mod body_limit;

#[cfg(feature = "compression")]
pub mod compression;

#[cfg(feature = "cookies")]
pub mod cookies {
    pub use tower_cookies::*;
}

#[cfg(feature = "cors")]
pub mod cors;

#[cfg(feature = "helmet")]
pub mod helmet;

#[cfg(feature = "not-found")]
pub mod not_found;

pub mod prelude;

#[cfg(any(feature = "body-limit", feature = "not-found", feature = "req-timeout"))]
use axum::{body::Body, response::Response};
#[cfg(any(feature = "body-limit", feature = "not-found", feature = "req-timeout"))]
use tower::{ServiceBuilder, util::MapResponseLayer as TowerMapResponseLayer};
#[cfg(any(feature = "body-limit", feature = "not-found", feature = "req-timeout"))]
use tower_layer::{Identity, Stack};

#[cfg(any(feature = "body-limit", feature = "not-found", feature = "req-timeout"))]
pub(crate) type ServiceLayer<Inner, Outer> = ServiceBuilder<Stack<Inner, Stack<Outer, Identity>>>;

#[cfg(any(feature = "body-limit", feature = "not-found", feature = "req-timeout"))]
pub(crate) type MapResponseLayer = TowerMapResponseLayer<ResponseFnMapper>;
#[cfg(any(feature = "body-limit", feature = "not-found", feature = "req-timeout"))]
pub(crate) type ResponseFnMapper = fn(Response<Body>) -> Response<Body>;

#[cfg(feature = "request-id")]
pub mod request_id;

#[cfg(feature = "servedir")]
pub mod servedir;

#[cfg(feature = "tracing")]
pub mod tracing;

#[cfg(feature = "req-timeout")]
pub mod timeout;

pub trait DisplayConfig {
    fn display(&self);
}
