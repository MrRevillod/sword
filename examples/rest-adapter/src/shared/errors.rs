use sword::prelude::*;
use thiserror::Error as ThisError;

pub type AppResult<T> = Result<T, AppError>;

#[derive(Debug, ThisError, HttpError)]
pub enum AppError {
    #[tracing(error)]
    #[http(code = 500)]
    #[error("Database error occurred: {0}")]
    DatabaseError(#[from] sqlx::Error),

    #[tracing(error)]
    #[http(code = 500)]
    #[error("Hasher error occurred: {0}")]
    HasherError(#[from] bcrypt::BcryptError),

    #[tracing(warn)]
    #[http(code = 404)]
    #[error("Not found error: {message}")]
    NotFoundError { message: String },

    #[tracing(error)]
    #[error("Conflict error")]
    #[http(code = 409, message = "User with {field} '{value}' already exists")]
    UserConflictError { field: String, value: String },
}
