use axum_responses::thiserror::Error;

#[cfg(feature = "validator")]
use serde_json::Value;

#[derive(Debug, Error)]
pub enum RequestError {
    #[error("Failed to parse request: {message}")]
    ParseError { message: String, details: String },

    #[error("Deserialization error: {error}")]
    DeserializationError {
        message: &'static str,
        error: String,

        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
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
    pub fn parse_error(
        message: impl Into<String>,
        details: impl Into<String>,
    ) -> Self {
        RequestError::ParseError {
            message: message.into(),
            details: details.into(),
        }
    }

    #[cfg(feature = "validator")]
    pub fn validator_error(message: &'static str, details: Value) -> Self {
        RequestError::ValidatorError { message, details }
    }

    pub fn unsupported_media_type(message: &'static str) -> Self {
        RequestError::UnsupportedMediaType { message }
    }

    pub fn deserialization_error(
        message: &'static str,
        error: String,
        source: Box<dyn std::error::Error + Send + Sync>,
    ) -> Self {
        RequestError::DeserializationError {
            message,
            error,
            source,
        }
    }
}
