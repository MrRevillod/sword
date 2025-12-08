#[cfg(feature = "validator")]
mod formatting;

pub use axum_responses::{
    ContentDisposition, File, HttpError, Redirect, thiserror::Error,
};

pub use axum_responses::{JsonResponse, JsonResponseBody};

pub type HttpResult<T> = Result<T, JsonResponse>;

#[cfg(feature = "validator")]
pub use formatting::format_validator_errors;

use crate::core::DependencyInjectionError;

impl From<DependencyInjectionError> for JsonResponse {
    fn from(error: DependencyInjectionError) -> Self {
        match error {
            DependencyInjectionError::BuildFailed { type_name, reason } => {
                eprintln!("Failed to build dependency '{type_name}': {reason}");
                JsonResponse::InternalServerError().message("Internal server error")
            }
            DependencyInjectionError::DependencyNotFound { type_name } => {
                eprintln!("Dependency '{type_name}' not found in container");
                JsonResponse::InternalServerError()
                    .message("Service configuration error")
            }
            DependencyInjectionError::ConfigInjectionError { source } => {
                eprintln!("Failed to inject config: {source}");
                JsonResponse::InternalServerError().message("Configuration error")
            }
            DependencyInjectionError::CircularDependency { type_name } => {
                eprintln!("Circular dependency detected involving '{type_name}'");
                JsonResponse::InternalServerError()
                    .message("Dependency injection error")
            }
        }
    }
}
