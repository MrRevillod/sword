use thiserror::Error;

#[derive(Debug, Error)]
pub enum RequestError {
    #[error("Failed to parse request: {0}")]
    ParseError(&'static str, String),

    #[cfg(feature = "validator")]
    #[error("Failed to validate request")]
    ValidatorError(&'static str, validator::ValidationErrors),

    #[error("Request body is empty")]
    BodyIsEmpty(&'static str),

    #[error("Request body is too large")]
    BodyTooLarge,

    #[error("Unsupported media type: {0}")]
    UnsupportedMediaType(String),

    #[error("Internal server error: {0}")]
    InternalError(String),
}
