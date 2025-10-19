use crate::{
    core::{ApplicationConfig, Config, State},
    web::{HttpResponse, Request, RequestError},
};

use axum::{
    body::{Body, to_bytes},
    extract::{FromRef, FromRequest, Path, Request as AxumReq},
    http::{HeaderName, HeaderValue},
};

use http_body_util::LengthLimitError;
use std::collections::HashMap;

/// Implementation of `FromRequest` for `Context`.
///
/// Allows `Context` to be automatically extracted from HTTP requests
/// in Axum handlers, providing easy access to parameters, headers, body, and state.
impl<S> FromRequest<S> for Request
where
    S: Send + Sync + 'static,
    State: FromRef<S>,
{
    type Rejection = HttpResponse;

    async fn from_request(req: AxumReq, state: &S) -> Result<Self, Self::Rejection> {
        let (mut parts, body) = req.into_parts();

        let mut params = HashMap::new();

        let path_result = {
            use axum::extract::OptionalFromRequestParts;
            Path::<HashMap<String, String>>::from_request_parts(&mut parts, &())
                .await
        };

        if let Ok(Some(path_params)) = path_result {
            params.extend(path_params.0);
        }

        let state = State::from_ref(state);

        let body_limit = state
            .get::<Config>()?
            .get::<ApplicationConfig>()
            .map(|app_config| app_config.body_limit.parsed)
            .unwrap_or(usize::MAX);

        let body_bytes = to_bytes(body, body_limit).await.map_err(|err| {
            let mut current_error: &dyn std::error::Error = &err;

            loop {
                if current_error.is::<LengthLimitError>() {
                    return RequestError::BodyTooLarge;
                }

                match std::error::Error::source(current_error) {
                    Some(source) => current_error = source,
                    None => break,
                }
            }

            RequestError::ParseError(
                "Failed to read request body",
                format!("Error reading body: {err}"),
            )
        })?;

        let mut headers = HashMap::new();

        for (key, value) in &parts.headers {
            if let Ok(value_str) = value.to_str() {
                headers.insert(key.to_string(), value_str.to_string());
            }
        }

        Ok(Self {
            params,
            body_bytes,
            method: parts.method,
            headers,
            uri: parts.uri,
            extensions: parts.extensions,
        })
    }
}

/// Implementation of conversion from `Context` to `AxumRequest`.
///
/// Allows converting a `Context` back to an Axum request,
/// preserving headers, method, URI, body, and extensions.
impl TryFrom<Request> for AxumReq {
    type Error = RequestError;

    fn try_from(req: Request) -> Result<Self, Self::Error> {
        let mut builder = AxumReq::builder().method(req.method).uri(req.uri);

        for (key, value) in req.headers {
            if let (Ok(header_name), Ok(header_value)) =
                (key.parse::<HeaderName>(), value.parse::<HeaderValue>())
            {
                builder = builder.header(header_name, header_value);
            }
        }

        let body = Body::from(req.body_bytes);

        let mut request = builder.body(body).map_err(|_| {
            RequestError::ParseError(
                "Failed to build axum request",
                "Error building request".to_string(),
            )
        })?;

        *request.extensions_mut() = req.extensions;

        Ok(request)
    }
}
