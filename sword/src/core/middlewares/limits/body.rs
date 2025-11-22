use std::str::FromStr;

use axum_responses::http::HttpResponse;
use byte_unit::Byte;
use serde::{Deserialize, Serialize};
use tower::layer::util::Stack;
use tower::{ServiceBuilder, layer::util::Identity};
use tower_http::limit::RequestBodyLimitLayer;

use axum::{
    body::Body,
    response::{IntoResponse, Response},
};

use tower::util::MapResponseLayer;

use crate::core::middlewares::ResponseFnMapper;

pub struct BodyLimitLayer;

type BodyLimitLayerType = ServiceBuilder<
    Stack<
        MapResponseLayer<ResponseFnMapper>,
        Stack<RequestBodyLimitLayer, Identity>,
    >,
>;

impl BodyLimitLayer {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(limit: usize) -> BodyLimitLayerType {
        fn map_body_limit_response(r: Response<Body>) -> Response<Body> {
            if r.status().as_u16() != 413 {
                return r;
            }

            HttpResponse::PayloadTooLarge()
            .message(
                "The request body exceeds the maximum allowed size by the server",
            )
            .into_response()
        }

        ServiceBuilder::new()
            .layer(RequestBodyLimitLayer::new(limit))
            .map_response(
                map_body_limit_response as fn(Response<Body>) -> Response<Body>,
            )
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct BodyLimit {
    pub raw: String,
    pub parsed: usize,
}

impl Default for BodyLimit {
    fn default() -> Self {
        BodyLimit {
            raw: "10MB".to_string(),
            parsed: Byte::from_str("10MB").unwrap().as_u64() as usize,
        }
    }
}

impl<'de> Deserialize<'de> for BodyLimit {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::{Error, MapAccess, Visitor};
        use std::fmt;

        struct BodyLimitVisitor;

        impl<'de> Visitor<'de> for BodyLimitVisitor {
            type Value = BodyLimit;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str(
                    "a string like \"10MB\" or an object with raw and parsed fields",
                )
            }

            // Deserialize from a string (from TOML config)
            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: Error,
            {
                let parsed = Byte::from_str(value)
                    .map(|b| b.as_u64() as usize)
                    .map_err(Error::custom)?;

                Ok(BodyLimit {
                    raw: value.to_string(),
                    parsed,
                })
            }

            // Deserialize from a map/object (from JSON serialization)
            fn visit_map<M>(self, mut map: M) -> Result<Self::Value, M::Error>
            where
                M: MapAccess<'de>,
            {
                let mut raw = None;
                let mut parsed = None;

                while let Some(key) = map.next_key::<String>()? {
                    match key.as_str() {
                        "raw" => raw = Some(map.next_value()?),
                        "parsed" => parsed = Some(map.next_value()?),
                        _ => {
                            let _: serde::de::IgnoredAny = map.next_value()?;
                        }
                    }
                }

                Ok(BodyLimit {
                    raw: raw.ok_or_else(|| Error::missing_field("raw"))?,
                    parsed: parsed.ok_or_else(|| Error::missing_field("parsed"))?,
                })
            }
        }

        deserializer.deserialize_any(BodyLimitVisitor)
    }
}
