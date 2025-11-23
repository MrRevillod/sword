use std::time::Duration;

use axum::http::HeaderValue;
use serde::{Deserialize, Serialize};
use tower_http::cors::CorsLayer as TowerCorsLayer;

#[derive(Clone, Debug, Deserialize, Serialize, Default)]
pub struct CorsConfig {
    pub allow_origins: Option<Vec<String>>,
    pub allow_methods: Option<Vec<String>>,
    pub allow_headers: Option<Vec<String>>,
    pub allow_credentials: Option<bool>,
    pub max_age: Option<u64>,

    #[serde(default = "default_display")]
    pub display: bool,
}

fn default_display() -> bool {
    false
}

impl CorsConfig {
    pub fn display(&self) {
        use console::style;

        println!();
        println!("{}", style("CORS Configuration:").bold());

        if let Some(origins) = &self.allow_origins {
            println!("  ↳  Allowed Origins: {:?}", origins);
        }
        if let Some(methods) = &self.allow_methods {
            println!("  ↳  Allowed Methods: {:?}", methods);
        }
        if let Some(headers) = &self.allow_headers {
            println!("  ↳  Allowed Headers: {:?}", headers);
        }
        if let Some(credentials) = self.allow_credentials {
            println!("  ↳  Allow Credentials: {}", credentials);
        }
        if let Some(max_age) = self.max_age {
            println!("  ↳  Max Age: {}s", max_age);
        }
    }
}

pub(crate) struct CorsLayer;

impl CorsLayer {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(config: &CorsConfig) -> TowerCorsLayer {
        let mut layer = TowerCorsLayer::new();

        if let Some(allow_credentials) = config.allow_credentials {
            layer = layer.allow_credentials(allow_credentials);
        }

        if let Some(origin) = &config.allow_origins {
            let parsed_origin: Vec<axum::http::header::HeaderValue> = origin
                .iter()
                .filter_map(|o| HeaderValue::from_str(o).ok())
                .collect();

            layer = layer.allow_origin(parsed_origin);
        }

        if let Some(methods) = &config.allow_methods {
            let parsed_methods: Vec<axum::http::Method> =
                methods.iter().filter_map(|m| m.parse().ok()).collect();

            layer = layer.allow_methods(parsed_methods);
        }

        if let Some(headers) = &config.allow_headers {
            let parsed_headers: Vec<axum::http::header::HeaderName> =
                headers.iter().filter_map(|h| h.parse().ok()).collect();

            layer = layer.allow_headers(parsed_headers);
        }

        if let Some(max_age) = config.max_age {
            layer = layer.max_age(Duration::from_secs(max_age));
        }

        layer
    }
}
