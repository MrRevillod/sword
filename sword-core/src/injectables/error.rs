use crate::ConfigError;
use axum_responses::JsonResponse;
use thiserror::Error;

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

    #[error(
        "Circular dependency detected\n  ↳ Ensure there are no cycles in your dependencies"
    )]
    CircularDependency,
}

impl From<DependencyInjectionError> for JsonResponse {
    fn from(error: DependencyInjectionError) -> Self {
        tracing::error!("Dependency injection error: {}", error);
        JsonResponse::InternalServerError()
    }
}

impl From<ConfigError> for JsonResponse {
    fn from(error: ConfigError) -> Self {
        match error {
            ConfigError::KeyNotFound { key } => {
                tracing::error!("Configuration key not found: {key}");
                JsonResponse::InternalServerError()
            }

            _ => JsonResponse::InternalServerError(),
        }
    }
}
