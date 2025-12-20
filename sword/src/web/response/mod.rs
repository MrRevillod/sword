#[cfg(feature = "validator")]
mod formatting;

pub use axum_responses::{ContentDisposition, File, Redirect};
pub use axum_responses::{JsonResponse, JsonResponseBody};
pub use sword_macros::HttpError;

pub type HttpResult<T = JsonResponse> = Result<T, JsonResponse>;

#[cfg(feature = "validator")]
pub use formatting::format_validator_errors;

use crate::core::{ConfigError, DependencyInjectionError};
use crate::web::RequestError;

impl From<RequestError> for JsonResponse {
    fn from(error: RequestError) -> JsonResponse {
        match error {
            RequestError::ParseError { message, details } => {
                tracing::error!(details = ?details,  "Request parse error: {message}");
                JsonResponse::BadRequest().message(message).error(details)
            }

            #[cfg(feature = "validator")]
            RequestError::ValidatorError { message, details } => {
                tracing::error!(details = ?details,  "Request validation error: {message}");
                JsonResponse::BadRequest().message(message).errors(details)
            }

            RequestError::BodyIsEmpty => {
                JsonResponse::BadRequest().message("Request body is empty")
            }
            RequestError::BodyTooLarge => JsonResponse::PayloadTooLarge().message(
                "The request body exceeds the maximum allowed size by the server",
            ),

            RequestError::UnsupportedMediaType { message } => {
                JsonResponse::UnsupportedMediaType().message(message)
            }

            RequestError::DeserializationError {
                message,
                error,
                source,
            } => {
                tracing::error!(source = %source, "Request deserialization error: {message}");
                JsonResponse::BadRequest().message(message).error(error)
            }
        }
    }
}

impl From<DependencyInjectionError> for JsonResponse {
    fn from(error: DependencyInjectionError) -> Self {
        match error {
            DependencyInjectionError::BuildFailed { .. } => {
                JsonResponse::InternalServerError().message("Internal server error")
            }
            DependencyInjectionError::DependencyNotFound { .. } => {
                JsonResponse::InternalServerError()
                    .message("Service configuration error")
            }
            DependencyInjectionError::ConfigInjectionError { .. } => {
                JsonResponse::InternalServerError().message("Configuration error")
            }
            DependencyInjectionError::CircularDependency { .. } => {
                JsonResponse::InternalServerError()
                    .message("Dependency injection error")
            }
        }
    }
}

impl From<ConfigError> for JsonResponse {
    fn from(error: ConfigError) -> Self {
        match error {
            ConfigError::KeyNotFound { key } => {
                tracing::error!("Configuration key not found: {key}");
                JsonResponse::InternalServerError().message("Configuration error")
            }

            _ => JsonResponse::InternalServerError()
                .message("An error occurred while processing the app configuration"),
        }
    }
}
