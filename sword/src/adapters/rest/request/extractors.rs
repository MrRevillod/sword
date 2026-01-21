use super::{super::JsonResponse, FromRequest, FromRequestParts, RequestError};
use axum::http::request::Parts;
use axum::{body::to_bytes, extract::Request as AxumReq};
use http_body_util::LengthLimitError;
use serde::de::DeserializeOwned;
use sword_core::{State, layers::MiddlewaresConfig};

// Re-export axum's FromRequest and FromRequestParts for implementing compatibility
use axum::extract::FromRequest as AxumFromRequest;
use axum::extract::FromRequestParts as AxumFromRequestParts;

/// Extractor for JSON body deserialization.
///
/// Similar to `req.body()` but as a standalone extractor.
/// Expects `Content-Type: application/json`.
#[derive(Debug, Clone)]
pub struct Json<T>(pub T);

impl<T> FromRequest for Json<T>
where
    T: DeserializeOwned + Send + 'static,
{
    type Rejection = JsonResponse;

    async fn from_request(
        req: AxumReq,
        state: &State,
    ) -> Result<Self, Self::Rejection> {
        let (parts, body) = req.into_parts();

        let is_json = parts
            .headers
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .map(|ct| ct.starts_with("application/json"))
            .unwrap_or(false);

        if !is_json {
            return Err(RequestError::unsupported_media_type(
                "Expected Content-Type to be application/json",
            ))?;
        }

        let body_limit = state
            .get::<MiddlewaresConfig>()
            .unwrap_or_default()
            .body_limit
            .max_size
            .parsed;

        if let Some(content_length) = parts.headers.get("content-length") {
            let cl_str = content_length.to_str().map_err(|_| {
                RequestError::parse_error(
                    "Invalid Content-Length header",
                    "Header contains invalid format",
                )
            })?;

            let size = cl_str.parse::<usize>().map_err(|_| {
                RequestError::parse_error(
                    "Invalid Content-Length header",
                    "Header value must be a valid number",
                )
            })?;

            if size > body_limit {
                return Err(RequestError::BodyTooLarge.into());
            }
        }

        let body_bytes = to_bytes(body, body_limit).await.map_err(|err| {
            if err.into_inner().is::<LengthLimitError>() {
                return RequestError::BodyTooLarge;
            }

            RequestError::parse_error(
                "Failed to read request body",
                "Error reading body".to_string(),
            )
        })?;

        if body_bytes.is_empty() {
            return Err(RequestError::BodyIsEmpty.into());
        }

        let value = serde_json::from_slice(&body_bytes).map_err(|e| {
            RequestError::deserialization_error(
                "Invalid request body",
                "Failed to deserialize request body to the required type.".into(),
                e.into(),
            )
        })?;

        Ok(Json(value))
    }
}

impl<T> AxumFromRequest<State> for Json<T>
where
    T: DeserializeOwned + Send + 'static,
{
    type Rejection = JsonResponse;

    async fn from_request(
        req: AxumReq,
        state: &State,
    ) -> Result<Self, Self::Rejection> {
        <Self as FromRequest>::from_request(req, state).await
    }
}

/// Extractor for URL query parameters.
///
/// Similar to `req.query()` but as a standalone extractor.
/// Returns `None` if no query parameters are present.
#[derive(Debug, Clone)]
pub struct Query<T>(pub Option<T>);

impl<T> FromRequestParts for Query<T>
where
    T: DeserializeOwned + Send + 'static,
{
    type Rejection = JsonResponse;

    async fn from_request_parts(
        parts: &mut Parts,
        _state: &State,
    ) -> Result<Self, Self::Rejection> {
        let Some(query_string) = parts.uri.query() else {
            return Ok(Query(None));
        };

        if query_string.is_empty() {
            return Ok(Query(None));
        }

        let deserializer = serde_urlencoded::Deserializer::new(
            form_urlencoded::parse(query_string.as_bytes()),
        );

        let deserialized = T::deserialize(deserializer).map_err(|e| {
            RequestError::deserialization_error(
                "Invalid query parameters",
                "Failed to deserialize query params to the required type.".into(),
                e.into(),
            )
        })?;

        Ok(Query(Some(deserialized)))
    }
}

// Implementación del trait de Axum para Query
impl<T> AxumFromRequestParts<State> for Query<T>
where
    T: DeserializeOwned + Send + 'static,
{
    type Rejection = JsonResponse;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &State,
    ) -> Result<Self, Self::Rejection> {
        <Self as FromRequestParts>::from_request_parts(parts, state).await
    }
}

/// Extractor for path parameters.
///
/// Similar to `req.param()` but extracts all path parameters at once.
/// Note: Use `PathParams` as the name to avoid confusion with std::path::Path
#[derive(Debug, Clone)]
pub struct PathParams<T>(pub T);

impl<T> FromRequestParts for PathParams<T>
where
    T: DeserializeOwned + Send + 'static,
{
    type Rejection = JsonResponse;

    async fn from_request_parts(
        parts: &mut Parts,
        _state: &State,
    ) -> Result<Self, Self::Rejection> {
        use axum::RequestPartsExt;

        let path_params =
            parts
                .extract::<axum::extract::Path<T>>()
                .await
                .map_err(|e| {
                    let message =
                        format!("Failed to extract path parameters: {}", e);
                    JsonResponse::BadRequest().message(message)
                })?;

        Ok(PathParams(path_params.0))
    }
}

// Implementación del trait de Axum para PathParams
impl<T> AxumFromRequestParts<State> for PathParams<T>
where
    T: DeserializeOwned + Send + 'static,
{
    type Rejection = JsonResponse;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &State,
    ) -> Result<Self, Self::Rejection> {
        <Self as FromRequestParts>::from_request_parts(parts, state).await
    }
}

// Implementar FromRequestParts directamente en http::Method
impl FromRequestParts for axum::http::Method {
    type Rejection = JsonResponse;

    async fn from_request_parts(
        parts: &mut Parts,
        _state: &State,
    ) -> Result<Self, Self::Rejection> {
        Ok(parts.method.clone())
    }
}

// Implementar FromRequestParts directamente en http::Uri
impl FromRequestParts for axum::http::Uri {
    type Rejection = JsonResponse;

    async fn from_request_parts(
        parts: &mut Parts,
        _state: &State,
    ) -> Result<Self, Self::Rejection> {
        Ok(parts.uri.clone())
    }
}

// Implementar FromRequestParts directamente en http::HeaderMap
impl FromRequestParts for axum::http::HeaderMap {
    type Rejection = JsonResponse;

    async fn from_request_parts(
        parts: &mut Parts,
        _state: &State,
    ) -> Result<Self, Self::Rejection> {
        Ok(parts.headers.clone())
    }
}

/// Extractor for a single header value by name.
#[derive(Debug, Clone)]
pub struct Header {
    pub name: String,
    pub value: Option<String>,
}

impl Header {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            value: None,
        }
    }
}

impl FromRequestParts for Header {
    type Rejection = JsonResponse;

    async fn from_request_parts(
        _parts: &mut Parts,
        _state: &State,
    ) -> Result<Self, Self::Rejection> {
        // This extractor is meant to be used manually by calling Header::new()
        // before extraction, which isn't ideal. Better to use Headers extractor
        // and get specific headers from it.
        Err(JsonResponse::InternalServerError().message(
            "Header extractor should not be used directly. Use Headers instead.",
        ))
    }
}

/// Extractor for request extensions.
#[derive(Debug, Clone)]
pub struct Extension<T: Clone + Send + Sync + 'static>(pub T);

impl<T: Clone + Send + Sync + 'static> FromRequestParts for Extension<T> {
    type Rejection = JsonResponse;

    async fn from_request_parts(
        parts: &mut Parts,
        _state: &State,
    ) -> Result<Self, Self::Rejection> {
        let value = parts.extensions.get::<T>().cloned().ok_or_else(|| {
            JsonResponse::InternalServerError().message(format!(
                "Extension of type {} not found",
                std::any::type_name::<T>()
            ))
        })?;

        Ok(Extension(value))
    }
}

// Implementar FromRequest directamente en axum::body::Bytes
impl FromRequest for axum::body::Bytes {
    type Rejection = JsonResponse;

    async fn from_request(
        req: AxumReq,
        state: &State,
    ) -> Result<Self, Self::Rejection> {
        let (parts, body) = req.into_parts();

        let body_limit = state
            .get::<MiddlewaresConfig>()
            .unwrap_or_default()
            .body_limit
            .max_size
            .parsed;

        if let Some(content_length) = parts.headers.get("content-length") {
            let cl_str = content_length.to_str().map_err(|_| {
                RequestError::parse_error(
                    "Invalid Content-Length header",
                    "Header contains invalid format",
                )
            })?;

            let size = cl_str.parse::<usize>().map_err(|_| {
                RequestError::parse_error(
                    "Invalid Content-Length header",
                    "Header value must be a valid number",
                )
            })?;

            if size > body_limit {
                return Err(RequestError::BodyTooLarge.into());
            }
        }

        let body_bytes = to_bytes(body, body_limit).await.map_err(|err| {
            if err.into_inner().is::<LengthLimitError>() {
                return RequestError::BodyTooLarge;
            }

            RequestError::parse_error(
                "Failed to read request body",
                "Error reading body".to_string(),
            )
        })?;

        Ok(body_bytes)
    }
}

// Nota: Usar PathParams<HashMap<String, String>> para extraer todos los parámetros de ruta

#[cfg(feature = "validation-validator")]
/// Extractor for JSON body with validation using `validator` crate.
///
/// Similar to `req.body_validator()` but as a standalone extractor.
#[derive(Debug, Clone)]
pub struct ValidatedJson<T>(pub T);

#[cfg(feature = "validation-validator")]
impl<T> FromRequest for ValidatedJson<T>
where
    T: DeserializeOwned + validator::Validate + Send + 'static,
{
    type Rejection = JsonResponse;

    async fn from_request(
        req: AxumReq,
        state: &State,
    ) -> Result<Self, Self::Rejection> {
        // First extract as Json, explicitly using our FromRequest trait
        let Json(value) = <Json<T> as FromRequest>::from_request(req, state).await?;

        // Then validate
        value.validate().map_err(|e| {
            use crate::adapters::rest::format_validator_errors;
            RequestError::validator_error(
                "Validation failed",
                format_validator_errors(e),
            )
        })?;

        Ok(ValidatedJson(value))
    }
}

#[cfg(feature = "validation-validator")]
/// Extractor for query parameters with validation.
///
/// Similar to `req.query_validator()` but as a standalone extractor.
#[derive(Debug, Clone)]
pub struct ValidatedQuery<T>(pub Option<T>);

#[cfg(feature = "validation-validator")]
impl<T> FromRequestParts for ValidatedQuery<T>
where
    T: DeserializeOwned + validator::Validate + Send + 'static,
{
    type Rejection = JsonResponse;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &State,
    ) -> Result<Self, Self::Rejection> {
        let Query(maybe_value) =
            <Query<T> as FromRequestParts>::from_request_parts(parts, state).await?;

        if let Some(value) = maybe_value {
            value.validate().map_err(|e| {
                use crate::adapters::rest::format_validator_errors;
                RequestError::validator_error(
                    "Validation failed",
                    format_validator_errors(e),
                )
            })?;

            Ok(ValidatedQuery(Some(value)))
        } else {
            Ok(ValidatedQuery(None))
        }
    }
}

#[cfg(feature = "validation-validator")]
/// Extractor for path parameters with validation.
#[derive(Debug, Clone)]
pub struct ValidatedPath<T>(pub T);

#[cfg(feature = "validation-validator")]
impl<T> FromRequestParts for ValidatedPath<T>
where
    T: DeserializeOwned + validator::Validate + Send + 'static,
{
    type Rejection = JsonResponse;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &State,
    ) -> Result<Self, Self::Rejection> {
        let PathParams::<T>(value) =
            <PathParams<T> as FromRequestParts>::from_request_parts(parts, state)
                .await?;

        value.validate().map_err(|e| {
            use crate::adapters::rest::format_validator_errors;
            RequestError::validator_error(
                "Validation failed",
                format_validator_errors(e),
            )
        })?;

        Ok(ValidatedPath(value))
    }
}
