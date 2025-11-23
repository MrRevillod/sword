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
    #[serde(default)]
    pub body_limit: BodyLimit,

    /// Request timeout configuration.
    #[serde(default)]
    pub request_timeout: TimeoutLimit,
}

impl LimitsMiddlewareConfig {
    pub fn display(&self) {
        let banner_top = "▪─────────────── ⚔ L I M I T S ⚔ ─────────────▪".white();
        let banner_bot = "▪──────────────── ⚔ ───────── ⚔ ──────────────▪".white();

        println!();
        println!("{banner_top}");

        if self.body_limit.enabled {
            println!("Max Body Size: {} ({})", self.body_limit.max_size.bright_green(), "enabled".bright_green());
        } else {
            println!("Max Body Size: {} ({})", self.body_limit.max_size.bright_yellow(), "disabled".bright_yellow());
        }

        if self.request_timeout.enabled {
            println!("Request Timeout: {} ({})", self.request_timeout.duration.bright_green(), "enabled".bright_green());
        } else {
            println!("Request Timeout: {} ({})", self.request_timeout.duration.bright_yellow(), "disabled".bright_yellow());
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
