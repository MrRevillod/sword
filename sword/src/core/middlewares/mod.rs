mod compression;
mod content_type;
mod cors;

#[cfg(feature = "helmet")]
pub mod helmet;

mod limits;
mod servedir;

pub(crate) use compression::*;
pub(crate) use content_type::ContentTypeCheck;
pub(crate) use cors::*;
pub(crate) use limits::*;
pub(crate) use servedir::*;

use axum::{body::Body, http::Response};

pub(self) type ResponseFnMapper = fn(Response<Body>) -> Response<Body>;
