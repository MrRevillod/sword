use thisconfig::ConfigError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DependencyInjectionError {
    #[error("Failed to build dependency '{type_name}': {reason}")]
    BuildFailed { type_name: String, reason: String },

    #[error("Dependency '{type_name}' not found in dependency container")]
    DependencyNotFound { type_name: String },

    #[error("Failed to inject config: {source}")]
    ConfigInjectionError {
        #[from]
        source: ConfigError,
    },

    #[error("Circular dependency detected in dependency container")]
    CircularDependency,
}
