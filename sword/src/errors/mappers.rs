use crate::{errors::*, web::HttpResponse};

#[cfg(feature = "validator")]
use crate::errors::formatting::format_validator_errors;

impl From<RequestError> for HttpResponse {
    fn from(error: RequestError) -> HttpResponse {
        match error {
            RequestError::ParseError(message, details) => {
                HttpResponse::BadRequest().message(message).error(details)
            }

            #[cfg(feature = "validator")]
            RequestError::ValidatorError(message, errors) => {
                HttpResponse::BadRequest()
                    .message(message)
                    .errors(format_validator_errors(errors))
            }

            RequestError::BodyIsEmpty(message) => {
                HttpResponse::BadRequest().message(message)
            }
            RequestError::BodyTooLarge => HttpResponse::PayloadTooLarge().message(
                "The request body exceeds the maximum allowed size by the server",
            ),

            RequestError::UnsupportedMediaType(message) => {
                HttpResponse::UnsupportedMediaType().message(message)
            }

            RequestError::InternalError(message) => {
                eprintln!("Internal server error: {message}");
                HttpResponse::InternalServerError().message("Internal server error")
            }
        }
    }
}

impl From<StateError> for HttpResponse {
    fn from(error: StateError) -> Self {
        match error {
            StateError::TypeNotFound { .. } => HttpResponse::InternalServerError(),
            StateError::LockError => HttpResponse::InternalServerError(),
        }
    }
}

impl From<DependencyInjectionError> for HttpResponse {
    fn from(error: DependencyInjectionError) -> Self {
        match error {
            DependencyInjectionError::BuildFailed { type_name, reason } => {
                eprintln!("Failed to build dependency '{type_name}': {reason}");
                HttpResponse::InternalServerError().message("Internal server error")
            }
            DependencyInjectionError::DependencyNotFound { type_name } => {
                eprintln!("Dependency '{type_name}' not found in container");
                HttpResponse::InternalServerError()
                    .message("Service configuration error")
            }
            DependencyInjectionError::StateError { type_name, source } => {
                eprintln!(
                    "State error while building '{type_name}': {}",
                    source.to_string()
                );
                HttpResponse::InternalServerError().message("Internal server error")
            }
            DependencyInjectionError::ConfigInjectionError { source } => {
                eprintln!("Failed to inject config: {}", source.to_string());
                HttpResponse::InternalServerError().message("Configuration error")
            }
        }
    }
}

impl From<ConfigError> for HttpResponse {
    fn from(error: ConfigError) -> Self {
        match error {
            ConfigError::DeserializeError(message) => {
                eprintln!("Configuration error: {message}");
                HttpResponse::InternalServerError().message("Configuration error")
            }
            ConfigError::KeyNotFound(key) => {
                let message = format!("Key '{key}' not found in configuration");
                eprintln!("{message}");
                HttpResponse::InternalServerError().message("Configuration error")
            }

            _ => HttpResponse::InternalServerError()
                .message("An error occurred while processing the app configuration"),
        }
    }
}
