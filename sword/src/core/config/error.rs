use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("Configuration file not found at path: {0}")]
    FileNotFound(&'static str),

    #[error("Failed to read configuration file: {0}")]
    ReadError(std::io::Error),

    #[error("Failed to interpolate environment variables in configuration: {0}")]
    InterpolationError(String),

    #[error("Configuration key '{0}' not found")]
    KeyNotFound(String),

    #[error(
        "Configuration value for key '{key}' is invalid: {value}. Reason: {reason}"
    )]
    InvalidValue {
        key: String,
        value: String,
        reason: String,
    },
    #[error("Failed to build configuration: {0}")]
    BuildError(String),

    #[error("Failed to deserialize configuration: {0}")]
    DeserializeError(String),

    #[error("Failed to parse configuration: {0}")]
    ParseError(String),

    #[error("Error getting configuration from application state: {0}")]
    GetConfigError(String),
}
