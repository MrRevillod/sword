mod builder;
mod config;

use crate::runtimes::web::WebRuntime;

use std::path::Path;
use sword_core::{Config, sword_error};

pub use builder::ApplicationBuilder;
pub use config::ApplicationConfig;

/// The main application struct that holds the runtime(s) and configuration.
///
/// `Application` is the core component of the Sword framework that manages
/// the web server, routing, and application configuration. It provides a
/// builder pattern for configuration and methods to run the application.
pub struct Application {
    web_runtime: WebRuntime,
    pub config: Config,
}

impl Application {
    pub(crate) fn new(web_runtime: WebRuntime, config: Config) -> Self {
        Self {
            web_runtime,
            config,
        }
    }

    /// Creates a new application builder for configuring the application.
    ///
    /// This is the starting point for creating a new Sword application.
    /// The builder pattern allows you to configure various aspects of the
    /// application before building the final `Application` instance.
    ///
    /// This function will panic if:
    /// - The configuration file `config/config.toml` cannot be found
    /// - The configuration file contains invalid TOML syntax
    /// - Environment variable interpolation fails
    /// - The configuration cannot be loaded for any other reason
    pub fn builder() -> ApplicationBuilder {
        ApplicationBuilder::new()
    }

    pub fn from_config_path<P: AsRef<Path>>(path: P) -> ApplicationBuilder {
        let config_path = path.as_ref().display().to_string();

        ApplicationBuilder::from_config(
            Config::builder()
                .add_required_file(path.as_ref())
                .build()
                .unwrap_or_else(|err| {
                    sword_error! {
                        title: "Failed to load configuration from custom path",
                        reason: err,
                        context: {
                            "path" => config_path,
                            "source" => "Application::from_config_path",
                        },
                        hints: ["Ensure the file exists and contains valid TOML"],
                    }
                }),
        )
    }

    /// Runs the application server.
    ///
    /// This method starts the web server and begins listening for incoming
    /// requests. It will bind to the host and port specified in the
    /// runtime configuration.
    pub async fn run(&self) {
        self.web_runtime.start().await;
    }

    /// Returns a clone of the internal Axum router for testing purposes.
    ///
    /// This method provides access to the underlying Axum router for integration
    /// testing with axum-test or similar tools.
    pub fn router(&self) -> axum::Router {
        self.web_runtime.router()
    }
}
