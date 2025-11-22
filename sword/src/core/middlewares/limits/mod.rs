mod body;
mod timeout;

pub(crate) use body::{BodyLimit, BodyLimitLayer};
pub(crate) use timeout::TimeoutLayer;

use crate::core::{ConfigItem, ConfigRegistrar};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct LimitsMiddlewareConfig {
    /// Limit for the maximum allowed size of request bodies in bytes.
    /// This can be specified as a human-readable string like "10MB", "500KB", etc.
    /// If not specified, defaults to 10MB.
    #[serde(default)]
    pub body: BodyLimit,

    /// Optional request timeout in seconds.
    /// If not specified, no timeout is applied.
    pub request_timeout: Option<u64>,
}

impl ConfigItem for LimitsMiddlewareConfig {
    fn toml_key() -> &'static str {
        "limits"
    }
}

const _: () = {
    inventory::submit! {
        ConfigRegistrar::new(|config, state| {
            LimitsMiddlewareConfig::register_in_state(config, state)
        })
    }
};
