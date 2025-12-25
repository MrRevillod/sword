use colored::Colorize;
use console::style;
use serde::{Deserialize, Serialize};

use crate::{
    core::{Config, ConfigError, State},
    internal::{ConfigItem, ConfigRegistrar},
};

/// Configuration structure for the Sword application.
///
/// This struct contains all the configuration options that can be specified
/// in the `config/config.toml` file under the `[application]` section.
#[derive(Debug, Deserialize, Clone, Serialize)]
#[serde(default)]
pub struct ApplicationConfig {
    /// The hostname or IP address to bind the server to.
    /// Defaults to "0.0.0.0" if not specified.
    pub host: String,

    /// The port number to bind the server to.
    /// Defaults to 8000 if not specified.
    pub port: u16,

    /// Whether to enable graceful shutdown of the server.
    /// If true, the server will finish processing ongoing requests
    /// before shutting down when a termination signal is received.
    ///
    /// If you want to use a custom signal handler, you can disable this
    /// and implement your own signal with the `run_with_graceful_shutdown` method.
    pub graceful_shutdown: bool,

    /// Optional name of the application.
    /// This can be used for logging or display purposes.
    pub name: Option<String>,

    /// Optional environment name (e.g., "development", "production").
    /// This can be used to alter behavior based on the environment.
    pub environment: Option<String>,

    /// Optional global prefix for all routes.
    pub global_prefix: Option<String>,
}

impl ApplicationConfig {
    pub fn display(&self) {
        let banner_top = "▪──────────────── ⚔ S W O R D ⚔ ──────────────▪".white();

        println!("\n{banner_top}");

        println!();
        if let Some(name) = &self.name {
            println!("{}", style(format!("{} Configuration:", name)).bold());
        } else {
            println!("{}", style("Application Configuration:").bold());
        }

        println!("  ↳  Host: {}", self.host);
        println!("  ↳  Port: {}", self.port);

        if self.graceful_shutdown {
            println!("  ↳  Graceful Shutdown");
        } else {
            println!("  ↳  {}", style("Graceful Shutdown: disabled").red());
        }

        if let Some(env) = &self.environment {
            println!("  ↳  Environment: {}", env);
        }
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

    fn register(config: &Config, state: &State) -> Result<(), ConfigError> {
        state.insert(config.get_or_default::<Self>());
        Ok(())
    }
}

const _: () = {
    inventory::submit! {
        ConfigRegistrar::new(|config, state| {
            ApplicationConfig::register(config, state)
        })
    }
};

impl Default for ApplicationConfig {
    fn default() -> Self {
        Self {
            host: "0.0.0.0".to_string(),
            port: 8000,
            graceful_shutdown: false,
            name: None,
            environment: None,
            global_prefix: None,
        }
    }
}
