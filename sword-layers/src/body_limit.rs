use crate::DisplayConfig;

use byte_unit::Byte;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use thisconfig::ByteConfig;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct BodyLimitConfig {
    /// The maximum allowed size for request bodies (e.g., "1MB", "500KB").
    #[serde(rename = "max-size")]
    pub max_size: ByteConfig,
    /// Whether to display the configuration details.
    pub display: bool,
}

use crate::{MapResponseLayer, ResponseFnMapper, ServiceLayer};

use axum::{
    body::Body,
    response::{IntoResponse, Response},
};
use axum_responses::JsonResponse;
use tower::ServiceBuilder;
use tower_http::limit::RequestBodyLimitLayer;

/// ### Body Limit Layer
///
/// This structs represents the Body Limit Layer which
/// restricts the size of incoming request bodies.
///
/// The layer is in fact a `ServiceBuilder` that applies a body size limit
/// and maps responses to `sword` standardized responses.
pub struct BodyLimitLayer;

impl BodyLimitLayer {
    pub fn new(
        config: &BodyLimitConfig,
    ) -> ServiceLayer<MapResponseLayer, RequestBodyLimitLayer> {
        fn map_body_limit_response(r: Response<Body>) -> Response<Body> {
            if r.status().as_u16() != 413 {
                return r;
            }

            JsonResponse::PayloadTooLarge()
                .message("The request body exceeds the maximum allowed size by the server")
                .into_response()
        }

        ServiceBuilder::new()
            .layer(RequestBodyLimitLayer::new(config.max_size.parsed))
            .map_response(map_body_limit_response as ResponseFnMapper)
    }
}

impl DisplayConfig for BodyLimitConfig {
    fn display(&self) {
        if self.display {
            tracing::info!(
                target: "sword.layers.body-limit",
                max_body_size = ?self.max_size.raw,
            );
        }
    }
}

impl Default for BodyLimitConfig {
    fn default() -> Self {
        let max_size = "10MB".to_string();
        let parsed = Byte::from_str(&max_size)
            .unwrap_or_else(|_| Byte::from_u64(10 * 1024 * 1024))
            .as_u64() as usize;

        BodyLimitConfig {
            display: true,
            max_size: ByteConfig {
                parsed,
                raw: max_size,
            },
        }
    }
}

#[derive(Clone)]
pub struct BodyLimitValue(pub usize);

impl From<BodyLimitConfig> for BodyLimitValue {
    fn from(config: BodyLimitConfig) -> Self {
        BodyLimitValue(config.max_size.parsed)
    }
}

impl Default for BodyLimitValue {
    fn default() -> Self {
        BodyLimitValue(10 * 1024 * 1024) // Default to 10MB
    }
}
