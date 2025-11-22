mod body;
mod timeout;

use crate::core::{ConfigItem, ConfigRegistrar};
use colored::Colorize;
use serde::{Deserialize, Serialize};

pub(crate) use body::{BodyLimit, BodyLimitLayer};
pub(crate) use timeout::{TimeoutLayer, TimeoutLimit};

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct LimitsMiddlewareConfig {
    /// Limit for the maximum allowed size of request bodies in bytes.
    /// This can be specified as a human-readable string like "10MB", "500KB", etc.
    /// If not specified, defaults to 10MB.
    #[serde(default)]
    pub body: BodyLimit,

    /// Optional request timeout in seconds.
    /// If not specified, no timeout is applied.
    pub request_timeout: Option<TimeoutLimit>,
}

impl LimitsMiddlewareConfig {
    pub fn display(&self) {
        let banner_top = "▪─────────────── ⚔ L I M I T S ⚔ ─────────────▪".white();
        let banner_bot = "▪──────────────── ⚔ ───────── ⚔ ──────────────▪".white();

        println!();
        println!("{banner_top}");

        println!("Max Body Size: {}", self.body.raw.bright_green());

        if let Some(timeout) = &self.request_timeout {
            println!("Request Timeout: {}", timeout.raw.bright_green());
        } else {
            println!("Request Timeout: {}", "none".bright_yellow());
        }

        println!("{banner_bot}");
    }
}

impl ConfigItem for LimitsMiddlewareConfig {
    fn toml_key() -> &'static str {
        "limits"
    }
}

const _: () = {
    inventory::submit! {
        ConfigRegistrar::new(|config, state| {
            LimitsMiddlewareConfig::register(config, state)
        })
    }
};
