use socketioxide::SendError;
use socketioxide_core::parser::ParseError;
use thiserror::Error;

#[cfg(feature = "validation-validator")]
use validator::ValidationErrors;

#[derive(Debug, Error)]
pub enum SocketError {
    #[cfg(feature = "validation-validator")]
    #[error("Socket IO Validation Error: {0}")]
    Validation(#[from] ValidationErrors),

    #[error("Socket IO Send Error: {0}")]
    Send(#[from] SendError),

    #[error("Socket IO parsing Error: {0}")]
    Parsing(#[from] ParseError),

    #[error("Socket IO error: {0}")]
    Socket(#[from] socketioxide::SocketError),
}
