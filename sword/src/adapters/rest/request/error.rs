use thiserror::Error;

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

    #[cfg(feature = "validation-validator")]
    #[error("Failed to validate request")]
    ValidatorError {
        message: &'static str,
        details: serde_json::Value,
    },

    #[error("Request body is empty")]
    BodyIsEmpty,

    #[error("Request body is too large")]
    BodyTooLarge,

    #[error("Unsupported media type: {message}")]
    UnsupportedMediaType { message: &'static str },

    #[error("Invalid header name: {0}")]
    InvalidHeaderName(String),

    #[error("Invalid header value: {0}")]
    InvalidHeaderValue(String),
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

    #[cfg(feature = "validation-validator")]
    pub fn validator_error(
        message: &'static str,
        details: serde_json::Value,
    ) -> Self {
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

#[cfg(feature = "multipart")]
impl From<axum::extract::multipart::MultipartRejection> for RequestError {
    fn from(err: axum::extract::multipart::MultipartRejection) -> Self {
        Self::parse_error("Failed to parse multipart form data", err.to_string())
    }
}

impl From<axum::extract::multipart::MultipartError> for RequestError {
    fn from(err: axum::extract::multipart::MultipartError) -> Self {
        Self::parse_error("Failed to parse multipart form data", err.to_string())
    }
}
