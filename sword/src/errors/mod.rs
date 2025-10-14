use thiserror::Error;

mod mappers;

#[cfg(feature = "validator")]
mod formatting;

#[derive(Debug, Error)]
pub enum ApplicationError {
    #[error("Failed to bind to address {address}: {source}")]
    BindFailed {
        address: String,
        #[source]
        source: std::io::Error,
    },
    #[error("Failed to start server: {source}")]
    ServerError {
        #[source]
        source: std::io::Error,
    },
    #[error("Config Error: {source}")]
    ConfigError {
        #[from]
        source: ConfigError,
    },
}

#[derive(Debug, Error)]
pub enum StateError {
    #[error(
        "State type not found - ensure it is registered in the application state"
    )]
    TypeNotFound { type_name: String },

    #[error("Failed to acquire lock on state")]
    LockError,
}

#[derive(Debug, Error)]
pub enum DependencyInjectionError {
    #[error("Failed to build dependency '{type_name}'\n   ↳ Reason: {reason}")]
    BuildFailed { type_name: String, reason: String },

    #[error(
        "Dependency '{type_name}' not found in dependency container\n   ↳ Ensure it's registered before use"
    )]
    DependencyNotFound { type_name: String },

    #[error("Failed to inject config: {source}")]
    ConfigInjectionError {
        #[from]
        source: ConfigError,
    },

    #[error("State error while building '{type_name}': {source}")]
    StateError {
        type_name: String,
        #[source]
        source: StateError,
    },

    #[error("Circular dependency detected involving '{type_name}'")]
    CircularDependency { type_name: String },
}

#[derive(Debug, Error)]
pub enum RequestError {
    #[error("Failed to parse request: {0}")]
    ParseError(&'static str, String),

    #[cfg(feature = "validator")]
    #[error("Failed to validate request")]
    ValidatorError(&'static str, validator::ValidationErrors),

    #[error("Request body is empty")]
    BodyIsEmpty(&'static str),

    #[error("Request body is too large")]
    BodyTooLarge,

    #[error("Unsupported media type: {0}")]
    UnsupportedMediaType(String),

    #[error("Internal server error: {0}")]
    InternalError(String),
}

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
