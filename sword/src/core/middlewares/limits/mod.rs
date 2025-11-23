mod body;
mod timeout;

use crate::core::{ConfigItem, ConfigRegistrar};
use serde::{Deserialize, Serialize};

pub(crate) use body::{BodyLimit, BodyLimitLayer};
pub(crate) use timeout::{TimeoutLayer, TimeoutLimit};

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct LimitsMiddlewareConfig {
    #[serde(default)]
    pub body_limit: BodyLimit,

    #[serde(default)]
    pub request_timeout: TimeoutLimit,
}

impl LimitsMiddlewareConfig {
    pub fn display(&self) {
        use console::style;

        println!();
        println!("{}", style("Limits Configuration:").bold());

        if self.body_limit.enabled {
            println!("  ↳  Max Body Size: {}", self.body_limit.max_size);
        } else {
            println!(
                "  ↳  {}",
                style(format!(
                    "Max Body Size: {} (disabled)",
                    self.body_limit.max_size
                ))
                .red()
            );
        }

        if self.request_timeout.enabled {
            println!("  ↳   Request Timeout: {}", self.request_timeout.duration);
        } else {
            println!(
                "  ↳  {}",
                style(format!(
                    "Request Timeout: {} (disabled)",
                    self.request_timeout.duration
                ))
                .red()
            );
        }
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
