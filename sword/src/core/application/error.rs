use thiserror::Error;

#[derive(Debug, Error)]
pub enum ApplicationError {
    #[error("Failed to bind to address {address}: {source}")]
    BindFailed {
        address: String,
        #[source]
        source: std::io::Error,
    },
    #[error("Failed to start server: {source}")]
    ServerError {
        #[source]
        source: std::io::Error,
    },
}
