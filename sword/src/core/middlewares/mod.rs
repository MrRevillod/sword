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
use serde::{Deserialize, Serialize};
pub(crate) use servedir::*;

use axum::{body::Body, http::Response};

use crate::core::{ConfigItem, ConfigRegistrar};

pub(self) type ResponseFnMapper = fn(Response<Body>) -> Response<Body>;

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct MiddlewaresConfig {
    #[serde(default)]
    pub body_limit: BodyLimit,

    #[serde(default)]
    pub request_timeout: TimeoutLimit,

    #[serde(default)]
    pub compression: Option<CompressionMiddlewareConfig>,

    #[serde(default)]
    pub cors: Option<CorsConfig>,

    #[serde(default)]
    pub serve_dir: Option<ServeDirConfig>,
}

impl ConfigItem for MiddlewaresConfig {
    fn toml_key() -> &'static str {
        "middlewares"
    }
}

const _: () = {
    inventory::submit! {
        ConfigRegistrar::new(|config, state| {
            MiddlewaresConfig::register(config, state)
        })
    }
};
