use sword::web::HttpResponse;
use thiserror::Error;

pub type AppResult<T> = Result<T, AppError>;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Database error occurred: {source}")]
    DatabaseError {
        #[from]
        source: sqlx::Error,
    },

    #[error("Hasher error occurred: {source}")]
    HasherError {
        #[from]
        source: bcrypt::BcryptError,
    },

    #[error("Database migration error occurred: {source}")]
    MigrationError {
        #[from]
        source: sqlx::migrate::MigrateError,
    },

    #[error("Not found error: {0}")]
    NotFoundError(&'static str),
}

impl From<AppError> for HttpResponse {
    fn from(error: AppError) -> Self {
        match error {
            AppError::NotFoundError(message) => {
                HttpResponse::NotFound().message(message)
            }
            _ => HttpResponse::InternalServerError().message(error.to_string()),
        }
    }
}
