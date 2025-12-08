use std::str::FromStr;

use axum_responses::JsonResponse;
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

            JsonResponse::PayloadTooLarge()
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
    pub enabled: bool,
    pub max_size: String,
    #[serde(skip)]
    pub parsed: usize,

    #[serde(default = "default_display")]
    pub display: bool,
}

fn default_display() -> bool {
    false
}

impl BodyLimit {
    pub fn display(&self) {
        if self.enabled {
            println!("  ↳  Max Body Size: {}", self.max_size);
        } else {
            println!("  ↳  Max Body Size: disabled");
        }
    }
}

impl Default for BodyLimit {
    fn default() -> Self {
        let max_size = "10MB".to_string();
        let parsed = Byte::from_str(&max_size).unwrap().as_u64() as usize;
        BodyLimit {
            enabled: true,
            max_size,
            parsed,
            display: default_display(),
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
                    "a map with 'enabled' (bool), 'max_size' (string), and optional 'display' (bool) fields",
                )
            }

            // Deserialize from a map/object
            fn visit_map<M>(self, mut map: M) -> Result<Self::Value, M::Error>
            where
                M: MapAccess<'de>,
            {
                let mut enabled = None;
                let mut max_size = None;
                let mut display = None;

                while let Some(key) = map.next_key::<String>()? {
                    match key.as_str() {
                        "enabled" => enabled = Some(map.next_value()?),
                        "max_size" => max_size = Some(map.next_value()?),
                        "display" => display = Some(map.next_value()?),
                        _ => {
                            let _: serde::de::IgnoredAny = map.next_value()?;
                        }
                    }
                }

                let enabled =
                    enabled.ok_or_else(|| Error::missing_field("enabled"))?;

                let max_size: String =
                    max_size.ok_or_else(|| Error::missing_field("max_size"))?;

                let parsed = Byte::from_str(&max_size)
                    .map(|b| b.as_u64() as usize)
                    .map_err(Error::custom)?;

                Ok(BodyLimit {
                    enabled,
                    max_size,
                    parsed,
                    display: display.unwrap_or_else(default_display),
                })
            }
        }

        deserializer.deserialize_any(BodyLimitVisitor)
    }
}
