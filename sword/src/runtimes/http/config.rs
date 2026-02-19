use console::style;
use serde::{Deserialize, Serialize};
use sword_core::{ConfigItem, ConfigRegistrar, inventory_submit};
use sword_layers::DisplayConfig;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpRuntimeConfig {
    /// The hostname or IP address to bind the server to. Defaults to "0.0.0.0"
    pub host: String,

    /// The port number to bind the server to. Defaults to 8000
    pub port: u16,

    /// Optional global prefix for all http routes.
    #[serde(rename = "http-router-prefix")]
    pub http_router_prefix: Option<String>,
}

impl DisplayConfig for HttpRuntimeConfig {
    fn display(&self) {
        println!();
        println!("{}", style("HTTP Server Configuration:").bold());
        println!("  ↳  Host: {}", self.host);
        println!("  ↳  Port: {}", self.port);

        if let Some(prefix) = &self.http_router_prefix {
            println!("  ↳  Http Router Prefix: {prefix}");
        }
    }
}

impl ConfigItem for HttpRuntimeConfig {
    fn key() -> &'static str {
        "http"
    }
}

inventory_submit! {[
    ConfigRegistrar::new(|state, config| {
        state.insert(config.get_or_default::<HttpRuntimeConfig>());
    })
]}

impl Default for HttpRuntimeConfig {
    fn default() -> Self {
        Self {
            host: "0.0.0.0".to_string(),
            port: 8000,
            http_router_prefix: None,
        }
    }
}
