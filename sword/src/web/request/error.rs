use axum_responses::{HttpError, thiserror::Error};

#[cfg(feature = "validator")]
use serde_json::Value;

#[derive(Debug, Error, HttpError)]
pub enum RequestError {
    #[tracing(error)]
    #[error("Failed to parse request: {message}")]
    #[http(code = 400, message = message, error = details)]
    ParseError {
        message: &'static str,
        details: String,
    },

    #[tracing(error)]
    #[error("Deserialization error: {error}")]
    #[http(code = 400, message = message, error = error)]
    DeserializationError {
        message: &'static str,
        error: String,
    },

    #[tracing(error)]
    #[cfg(feature = "validator")]
    #[error("Failed to validate request")]
    #[http(code = 400, message = message, errors = details)]
    ValidatorError {
        message: &'static str,
        details: Value,
    },

    #[tracing(error)]
    #[error("Request body is empty")]
    #[http(code = 400, message = "Request body is empty")]
    BodyIsEmpty,

    #[tracing(error)]
    #[error("Request body is too large")]
    #[http(
        code = 413,
        message = "The request body exceeds the maximum allowed size by the server"
    )]
    BodyTooLarge,

    #[tracing(error)]
    #[error("Unsupported media type: {message}")]
    #[http(code = 415, message = message)]
    UnsupportedMediaType { message: &'static str },
}

impl RequestError {
    pub fn parse_error(message: &'static str, details: String) -> Self {
        RequestError::ParseError { message, details }
    }

    #[cfg(feature = "validator")]
    pub fn validator_error(message: &'static str, details: Value) -> Self {
        RequestError::ValidatorError { message, details }
    }

    pub fn unsupported_media_type(message: &'static str) -> Self {
        RequestError::UnsupportedMediaType { message }
    }

    pub fn deserialization_error(message: &'static str, error: String) -> Self {
        RequestError::DeserializationError { message, error }
    }
}
