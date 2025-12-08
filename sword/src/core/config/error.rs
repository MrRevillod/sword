use axum_responses::HttpError;
use thiserror::Error;

#[derive(Debug, Error, HttpError)]
pub enum ConfigError {
    #[tracing(error)]
    #[http(code = 500)]
    #[error("Configuration file not found at config/config.toml")]
    FileNotFound,

    #[tracing(error)]
    #[http(code = 500)]
    #[error("Failed to read configuration file: {source}")]
    ReadError {
        #[from]
        source: std::io::Error,
    },

    #[tracing(error)]
    #[http(code = 500)]
    #[error("Environment variable interpolation error: {message}")]
    InterpolationError { message: String },

    #[tracing(error)]
    #[http(code = 500)]
    #[error("Configuration key '{key}' not found")]
    KeyNotFound { key: String },

    #[tracing(error)]
    #[http(code = 500)]
    #[error("Deserialization error: {source}")]
    DeserializeError {
        #[from]
        source: toml::de::Error,
    },
}

impl ConfigError {
    pub fn interpolation_error(message: String) -> Self {
        ConfigError::InterpolationError { message }
    }

    pub fn key_not_found(key: impl Into<String>) -> Self {
        ConfigError::KeyNotFound { key: key.into() }
    }
}
