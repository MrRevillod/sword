use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("Configuration file not found at config/config.toml")]
    FileNotFound,

    #[error("Failed to read configuration file: {source}")]
    ReadError {
        #[from]
        source: std::io::Error,
    },

    #[error("Environment variable interpolation error: {message}")]
    InterpolationError { message: String },

    #[error("Configuration key '{key}' not found")]
    KeyNotFound { key: String },

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
