use super::{super::JsonResponse, Request, RequestError};
use sword_core::{State, layers::MiddlewaresConfig};

use axum::{
    RequestPartsExt,
    body::to_bytes,
    extract::{
        FromRef, FromRequest as AxumFromRequest, Path, Request as AxumReq,
        rejection::PathRejection,
    },
    http::request::Parts,
    response::IntoResponse,
};

use http_body_util::LengthLimitError;
use std::collections::HashMap;

#[allow(async_fn_in_trait)]
/// Fixed-state version of `axum::extract::FromRequest` using sword's State.
///
/// This allows extractors to avoid the generic state parameter while still
/// leveraging Axum's extractor ecosystem.
pub trait FromRequest: Sized {
    type Rejection: IntoResponse;

    async fn from_request(
        req: AxumReq,
        state: &State,
    ) -> Result<Self, Self::Rejection>;
}

#[allow(async_fn_in_trait)]
/// Fixed-state version of `axum::extract::FromRequestParts` using sword's State.
pub trait FromRequestParts: Sized {
    type Rejection: IntoResponse;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &State,
    ) -> Result<Self, Self::Rejection>;
}

/// Implementation of `FromRequest` for `Request`.
///
/// Allows `Request` to be automatically extracted from HTTP requests
/// in Axum handlers, providing easy access to parameters, headers, body, and state.
impl FromRequest for Request {
    type Rejection = JsonResponse;

    async fn from_request(
        req: AxumReq,
        state: &State,
    ) -> Result<Self, Self::Rejection> {
        let (mut parts, body) = req.into_parts();

        let path_params = parts
            .extract::<Path<HashMap<String, String>>>()
            .await
            .map_err(|e| {
                let message = match e {
                    PathRejection::FailedToDeserializePathParams(_) => {
                        "Failed to deserialize path parameters".to_string()
                    }
                    PathRejection::MissingPathParams(m) => m.body_text().to_string(),
                    _ => "Failed to extract path parameters".to_string(),
                };

                JsonResponse::BadRequest().message(message)
            })?;

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

        Ok(Self {
            params: path_params.0,
            body_bytes,
            method: parts.method,
            headers: parts.headers,
            uri: parts.uri,
            extensions: parts.extensions,
            next: None,
        })
    }
}

/// Implementation of conversion from `Request` to `axum::extract::Request`.
///
/// Allows converting a `sword::web::Request` back to an Axum request,
/// preserving headers, method, URI, body, and extensions.
impl TryFrom<Request> for AxumReq {
    type Error = RequestError;

    fn try_from(req: Request) -> Result<Self, Self::Error> {
        use axum::body::Body;

        let mut builder = AxumReq::builder().method(req.method).uri(req.uri);

        for (key, value) in &req.headers {
            builder = builder.header(key, value);
        }

        let body = Body::from(req.body_bytes);

        let mut request = builder.body(body).map_err(|_| {
            RequestError::parse_error(
                "Failed to build axum request",
                "Error building request".to_string(),
            )
        })?;

        *request.extensions_mut() = req.extensions;

        Ok(request)
    }
}

/// Implementation for compatibility with Axum's generic state system
impl<S> AxumFromRequest<S> for Request
where
    S: Send + Sync + 'static,
    State: FromRef<S>,
{
    type Rejection = JsonResponse;

    async fn from_request(req: AxumReq, state: &S) -> Result<Self, Self::Rejection> {
        let state = State::from_ref(state);
        <Self as FromRequest>::from_request(req, &state).await
    }
}
