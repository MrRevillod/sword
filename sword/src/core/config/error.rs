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

    #[error("Failed to interpolate environment variables in configuration: {0}")]
    InterpolationError(String),

    #[error("Configuration key '{0}' not found")]
    KeyNotFound(String),

    #[error("Deserialization error: {source}")]
    DeserializeError {
        #[from]
        source: toml::de::Error,
    },
}
