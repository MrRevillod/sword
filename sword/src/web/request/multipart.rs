use axum::extract::FromRequest;
pub use axum::extract::multipart::*;
pub use bytes;

use crate::web::{Request, RequestError};

impl Request {
    /// Extracts multipart form data from the request.
    ///
    /// ### Errors
    /// Returns `RequestError::ParseError` if the multipart form data cannot be parsed.
    ///
    /// ### Example
    /// ```rust,ignore
    /// async fn upload(&self, ctx: Context) -> HttpResult<HttpResponse> {
    ///     let mut multipart = ctx.multipart().await?;
    ///     let mut field_names = Vec::new();
    ///
    ///     // Process each field in the multipart form data
    ///     // And ensure to handle errors appropriately
    ///     while let Some(field) = multipart.next_field().await.unwrap() {
    ///         field_names.push(field.name().unwrap_or("Uknown").to_string());
    ///     }
    ///
    ///     Ok(HttpResponse::Ok().data(field_names))
    /// }
    /// ```
    pub async fn multipart(&self) -> Result<Multipart, RequestError> {
        Ok(Multipart::from_request(self.clone().try_into()?, &()).await?)
    }
}

impl From<MultipartRejection> for RequestError {
    fn from(err: MultipartRejection) -> Self {
        Self::ParseError("Failed to parse multipart form data", err.to_string())
    }
}

impl From<MultipartError> for RequestError {
    fn from(err: MultipartError) -> Self {
        Self::ParseError("Failed to parse multipart form data", err.to_string())
    }
}
