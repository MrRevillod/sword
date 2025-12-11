use axum_responses::thiserror::Error;

#[cfg(feature = "validator")]
use serde_json::Value;

#[derive(Debug, Error)]
pub enum RequestError {
    #[error("Failed to parse request: {message}")]
    ParseError {
        message: &'static str,
        details: String,
    },

    #[error("Deserialization error: {error}")]
    DeserializationError {
        message: &'static str,
        error: String,
    },

    #[cfg(feature = "validator")]
    #[error("Failed to validate request")]
    ValidatorError {
        message: &'static str,
        details: Value,
    },

    #[error("Request body is empty")]
    BodyIsEmpty,

    #[error("Request body is too large")]
    BodyTooLarge,

    #[error("Unsupported media type: {message}")]
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
