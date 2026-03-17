use super::engines::web::WebApplicationConfig;
use serde::{Deserialize, Serialize};
use sword_core::{ConfigItem, ConfigRegistrar, inventory_submit};

/// Configuration structure for the Sword application.
///
/// This struct contains all the configuration options that can be specified
/// in the `config/config.toml` file under the `[application]` section.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ApplicationConfig {
    /// Optional name of the application. Defaults `None`.
    /// This can be used for logging or display purposes.
    pub name: Option<String>,

    /// Optional environment name (e.g., "development", "production").
    /// This can be used to alter behavior based on the environment. Defaults `None`.
    pub environment: Option<String>,

    /// Whether to enable graceful shutdown of the server.
    /// If true, the server will finish processing ongoing requests
    /// before shutting down when a termination signal is received.
    ///
    /// If you want to use a custom signal handler, you can disable this
    /// and implement your own signal with the `run_with_graceful_shutdown` method.
    /// Defaults `false`
    #[serde(rename = "graceful-shutdown")]
    pub graceful_shutdown: bool,

    /// Web server settings flattened into `[application]`.
    ///
    /// This keeps the TOML ergonomic while still using a dedicated struct
    /// for web-specific configuration.
    #[serde(flatten)]
    pub web: WebApplicationConfig,
}

impl ConfigItem for ApplicationConfig {
    fn key() -> &'static str {
        "application"
    }
}

inventory_submit! {[
    ConfigRegistrar::new(|state, config| {
        state.insert(config.get_or_default::<ApplicationConfig>());
    })
]}
