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
