use console::style;
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
}

impl ApplicationConfig {
    pub fn display(&self) {
        println!();
        if let Some(name) = &self.name {
            println!("{}", style(format!("{} Configuration:", name)).bold());
        } else {
            println!("{}", style("Application Configuration:").bold());
        }

        if self.graceful_shutdown {
            println!("  ↳  Graceful Shutdown: enabled");
        } else {
            println!("  ↳  {}", style("Graceful Shutdown: disabled").red());
        }

        if let Some(env) = &self.environment {
            println!("  ↳  Environment: {}", env);
        }
    }
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
