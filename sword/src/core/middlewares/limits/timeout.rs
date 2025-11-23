use axum::response::{IntoResponse, Response};
use axum_responses::http::HttpResponse;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tower::{ServiceBuilder, util::MapResponseLayer};
use tower_http::timeout::TimeoutLayer as TowerTimeoutLayer;
use tower_layer::{Identity, Stack};

use crate::core::middlewares::ResponseFnMapper;

pub struct TimeoutLayer;

type TimeoutLayerType = (
    TowerTimeoutLayer,
    ServiceBuilder<Stack<MapResponseLayer<ResponseFnMapper>, Identity>>,
);

impl TimeoutLayer {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(duration: Duration) -> TimeoutLayerType {
        let timeout_layer = TowerTimeoutLayer::new(duration);

        fn timeout_mapper(response: Response) -> Response {
            if response.status().as_u16() == 408 {
                return HttpResponse::RequestTimeout().into_response();
            }

            response
        }

        let response_layer =
            ServiceBuilder::new().map_response(timeout_mapper as ResponseFnMapper);

        (timeout_layer, response_layer)
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct TimeoutLimit {
    pub enabled: bool,
    pub duration: String,
    #[serde(skip)]
    pub parsed: Duration,
}

impl<'de> Deserialize<'de> for TimeoutLimit {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::{Error, MapAccess, Visitor};
        use std::fmt;

        struct TimeoutLimitVisitor;

        impl<'de> Visitor<'de> for TimeoutLimitVisitor {
            type Value = TimeoutLimit;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str(
                    "a map with 'enabled' (bool) and 'duration' (string) fields",
                )
            }

            // Deserialize from a map/object
            fn visit_map<M>(self, mut map: M) -> Result<Self::Value, M::Error>
            where
                M: MapAccess<'de>,
            {
                let mut enabled = None;
                let mut duration = None;

                while let Some(key) = map.next_key::<String>()? {
                    match key.as_str() {
                        "enabled" => enabled = Some(map.next_value()?),
                        "duration" => duration = Some(map.next_value()?),
                        _ => {
                            let _: serde::de::IgnoredAny = map.next_value()?;
                        }
                    }
                }

                let enabled =
                    enabled.ok_or_else(|| Error::missing_field("enabled"))?;

                let duration: String =
                    duration.ok_or_else(|| Error::missing_field("duration"))?;

                let parsed =
                    duration_str::parse(&duration).map_err(Error::custom)?;

                Ok(TimeoutLimit {
                    enabled,
                    duration,
                    parsed,
                })
            }
        }

        deserializer.deserialize_any(TimeoutLimitVisitor)
    }
}

impl Default for TimeoutLimit {
    fn default() -> Self {
        let duration = "30s".to_string();
        let parsed = duration_str::parse(&duration).unwrap();
        TimeoutLimit {
            enabled: false,
            duration,
            parsed,
        }
    }
}
