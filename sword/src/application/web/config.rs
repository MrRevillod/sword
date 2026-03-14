use console::style;
use serde::{Deserialize, Serialize};
use sword_core::{
    ConfigItem, ConfigRegistrar, inventory_submit, layers::DisplayConfig,
};
use sword_layers::prelude::{BodyLimitConfig, RequestTimeoutConfig};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebApplicationConfig {
    /// The hostname or IP address to bind the server to. Defaults to "0.0.0.0"
    pub host: String,

    /// The port number to bind the server to. Defaults to 8000
    pub port: u16,

    /// Optional global prefix for all web controller routes.
    #[serde(rename = "web-router-prefix")]
    pub web_router_prefix: Option<String>,

    /// Body limit policy for web request extraction.
    #[serde(rename = "body-limit")]
    pub body_limit: BodyLimitConfig,

    /// Request timeout policy applied to web controllers.
    #[serde(rename = "request-timeout")]
    pub request_timeout: RequestTimeoutConfig,
}

impl DisplayConfig for WebApplicationConfig {
    fn display(&self) {
        println!();
        println!("{}", style("HTTP Server Configuration:").bold());
        println!("  ↳  Host: {}", self.host);
        println!("  ↳  Port: {}", self.port);

        if let Some(prefix) = &self.web_router_prefix {
            println!("  ↳  Web Router Prefix: {prefix}");
        }

        self.body_limit.display();
        self.request_timeout.display();
    }
}

impl ConfigItem for WebApplicationConfig {
    fn key() -> &'static str {
        "server"
    }
}

inventory_submit! {[
    ConfigRegistrar::new(|state, config| {
        state.insert(config.get_or_default::<WebApplicationConfig>());
    })
]}

impl Default for WebApplicationConfig {
    fn default() -> Self {
        Self {
            host: "0.0.0.0".to_string(),
            port: 8000,
            web_router_prefix: None,
            body_limit: BodyLimitConfig::default(),
            request_timeout: RequestTimeoutConfig::default(),
        }
    }
}
