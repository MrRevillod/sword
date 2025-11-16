use std::str::FromStr;

use byte_unit::Byte;
use colored::Colorize;
use serde::{Deserialize, Serialize};

use crate::core::{ConfigItem, ConfigRegistrar};

/// Configuration structure for the Sword application.
///
/// This struct contains all the configuration options that can be specified
/// in the `config/config.toml` file under the `[application]` section.
///
/// ### Configuration File Example
///
/// ```toml,ignore
/// [application]
/// host = "127.0.0.1"
/// port = 3000
/// body_limit = "10MB"
/// request_timeout_seconds = 30
/// graceful_shutdown = true
/// ```
///
/// ### Environment Variable Interpolation
///
/// Configuration values support environment variable interpolation:
///
/// ```toml,ignore
/// [application]
/// host = "${HOST:127.0.0.1}"
/// port = "${PORT:3000}"
/// ```
#[derive(Debug, Deserialize, Clone, Serialize, Default)]
pub struct ApplicationConfig {
    /// The hostname or IP address to bind the server to.
    /// Defaults to "0.0.0.0" if not specified.
    #[serde(default = "default_host")]
    pub host: String,

    /// The port number to bind the server to.
    /// Defaults to 8000 if not specified.
    #[serde(default = "default_port")]
    pub port: u16,

    /// Maximum size of request bodies that the server will accept.
    /// Specified as a string with units (e.g., "10MB", "1GB").
    /// Parsed using the byte_unit crate for flexible size specification.
    pub body_limit: BodyLimit,

    /// Optional request timeout in seconds.
    /// If set, requests taking longer than this duration will
    /// be aborted and return a timeout error.
    ///
    /// If not set, there is no timeout.
    pub request_timeout_seconds: Option<u64>,

    /// Whether to enable graceful shutdown of the server.
    /// If true, the server will finish processing ongoing requests
    /// before shutting down when a termination signal is received.
    ///
    /// If you want to use a custom signal handler, you can disable this
    /// and implement your own signal with the `run_with_graceful_shutdown` method.
    #[serde(default = "default_graceful_shutdown")]
    pub graceful_shutdown: bool,

    /// Optional name of the application.
    /// This can be used for logging or display purposes.
    pub name: Option<String>,

    /// Optional environment name (e.g., "development", "production").
    /// This can be used to alter behavior based on the environment.
    pub environment: Option<String>,

    pub global_prefix: Option<String>,
}

impl ApplicationConfig {
    pub fn display(&self) {
        let banner_top = "▪──────────────── ⚔ S W O R D ⚔ ──────────────▪".white();
        let banner_bot = "▪──────────────── ⚔ ───────── ⚔ ──────────────▪".white();

        println!("\n{}", banner_top);

        if let Some(name) = &self.name {
            println!("Application: {}", name.bright_green());
        }

        println!("Host: {}", self.host);
        println!("Port: {}", self.port);
        println!("Request Size Limit: {}", self.body_limit.raw);

        let timeout_display = if let Some(timeout) = self.request_timeout_seconds {
            format!("{} seconds", timeout)
        } else {
            "disabled".dimmed().to_string()
        };

        println!("Timeout: {}", timeout_display);

        let shutdown_display = if self.graceful_shutdown {
            "enabled".bright_green()
        } else {
            "disabled".bright_red()
        };

        println!("Graceful Shutdown: {}", shutdown_display);

        if let Some(env) = &self.environment {
            println!("Environment: {}", env.bright_blue());
        }

        println!("{}", banner_bot);
    }
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct BodyLimit {
    pub raw: String,
    pub parsed: usize,
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

/// Implementation of the `ConfigItem` trait for `ApplicationConfig`.
///
/// This implementation allows the application configuration to be automatically
/// loaded from TOML files using the "application" key.
impl ConfigItem for ApplicationConfig {
    /// Returns the TOML key used to identify this configuration section.
    ///
    /// For `ApplicationConfig`, this returns "application", meaning the
    /// configuration should be under the `[application]` section in the TOML file.
    fn toml_key() -> &'static str {
        "application"
    }
}

const _: () = {
    inventory::submit! {
        ConfigRegistrar::new(|config, state| {
            ApplicationConfig::register_in_state(config, state)
        })
    }
};

fn default_host() -> String {
    "0.0.0.0".to_string()
}

fn default_port() -> u16 {
    8000
}

fn default_graceful_shutdown() -> bool {
    false
}
