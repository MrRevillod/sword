use std::time::Duration;

use axum::http::HeaderValue;
use serde::Deserialize;
use tower_http::cors::CorsLayer as TowerCorsLayer;

use crate::core::{Config, ConfigError, ConfigItem, ConfigRegistrar, State};

#[derive(Clone, Debug, Deserialize)]
pub struct CorsConfig {
    pub allow_origins: Option<Vec<String>>,
    pub allow_methods: Option<Vec<String>>,
    pub allow_headers: Option<Vec<String>>,
    pub allow_credentials: Option<bool>,
    pub max_age: Option<u64>,
}

pub struct CorsLayer {}

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

const _: () = {
    inventory::submit! {
        ConfigRegistrar::new(|config, state| {
            CorsConfig::register_in_state(config, state)
        })
    }
};

impl ConfigItem for CorsConfig {
    fn toml_key() -> &'static str {
        "cors"
    }

    fn register_in_state(config: &Config, state: &State) -> Result<(), ConfigError> {
        state.insert(config.get::<Self>()?).map_err(|_| {
            ConfigError::ParseError(format!(
                "Failed to register config '{}' in state",
                Self::toml_key()
            ))
        })
    }
}
