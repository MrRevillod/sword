use thiserror::Error;

use crate::core::{ConfigError, StateError};

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

impl DependencyInjectionError {
    pub fn state_error(type_name: &str, source: StateError) -> Self {
        DependencyInjectionError::StateError {
            type_name: type_name.to_string(),
            source,
        }
    }
}
