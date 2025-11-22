mod content_type;
mod cors;

#[cfg(feature = "helmet")]
pub mod helmet;
mod limits;

pub(crate) use content_type::ContentTypeCheck;
pub(crate) use cors::*;
pub(crate) use limits::*;

use axum::{body::Body, http::Response};

pub(crate) type ResponseFnMapper = fn(Response<Body>) -> Response<Body>;
