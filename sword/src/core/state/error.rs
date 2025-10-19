use thiserror::Error;

#[derive(Debug, Error)]
pub enum StateError {
    #[error(
        "State type not found - ensure it is registered in the application state"
    )]
    TypeNotFound { type_name: String },

    #[error("Failed to acquire lock on state")]
    LockError,
}
