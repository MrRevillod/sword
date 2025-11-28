use thiserror::Error;

use crate::core::ConfigError;

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

    #[error("Circular dependency detected involving '{type_name}'")]
    CircularDependency { type_name: String },
}
