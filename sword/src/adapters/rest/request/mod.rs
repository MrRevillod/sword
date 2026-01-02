mod error;
pub mod extract;
mod validator;

use super::interceptor::HttpInterceptorResult;
use axum::{
    body::Bytes as BodyBytes,
    http::{Extensions, HeaderMap, HeaderName, HeaderValue, Method, Uri},
    middleware::Next,
};
use axum_responses::JsonResponse;
use serde::de::DeserializeOwned;
use std::{collections::HashMap, str::FromStr};
use sword_core::layers::RequestId;

pub use sword_layers::cookies::{
    Cookies, Key, PrivateCookies, SignedCookies,
    cookie::{
        Cookie, CookieBuilder, Expiration, KeyError as CookieKeyError,
        ParseError as CookieParseError, SameSite,
    },
};

pub use error::*;

#[cfg(feature = "multipart")]
pub use bytes::*;

#[cfg(feature = "validation-validator")]
pub use validator::*;

/// Represents the incoming request in the Sword framework.
///
/// `Request` is the primary extractor for accessing request data in Sword applications.
/// It provides access to request parameters, body data, HTTP method, headers, URI,
#[derive(Debug, Clone)]
pub struct Request {
    params: HashMap<String, String>,
    body_bytes: BodyBytes,
    method: Method,
    headers: HeaderMap,
    uri: Uri,
    next: Option<Next>,
    /// Axum extensions for additional request metadata.
    pub extensions: Extensions,
}

impl Request {
    /// Gets the complete URI of the request as a string.
    ///
    /// ### Returns
    /// A `String` containing the complete request URI including
    /// the path and query parameters if any.
    pub fn uri(&self) -> String {
        self.uri.to_string()
    }

    /// Gets the HTTP method of the request.
    ///
    /// ### Returns
    /// A reference to the HTTP `Method` (GET, POST, PUT, DELETE, etc.).
    pub const fn method(&self) -> &Method {
        &self.method
    }

    /// Gets the value of a specific header by name.
    ///
    /// ### Arguments
    /// * `key` - The header name to get.
    ///
    /// ### Returns
    /// `Some(&str)` with the header value if it exists, `None` if not found.
    pub fn header(&self, key: &str) -> Option<&str> {
        self.headers.get(key).and_then(|value| value.to_str().ok())
    }

    /// Gets an immutable reference to all request headers.
    ///
    /// ### Returns
    /// A reference to `HashMap<String, String>` containing all request headers
    /// where the key is the header name and the value is its content.
    pub const fn headers(&self) -> &HeaderMap {
        &self.headers
    }

    /// Gets a mutable reference to all request headers.
    ///
    /// ### Returns
    /// A mutable reference to `HeaderMap` that allows modifying
    /// existing headers or adding new headers to the request.
    pub fn headers_mut(&mut self) -> &mut HeaderMap {
        &mut self.headers
    }

    /// Sets or updates the value of a header in the request.
    ///
    /// ### Arguments
    /// * `name` - The header name to set. Must implement `Into<String>`.
    /// * `value` - The header value to set. Must implement `Into<String>`.
    ///
    /// ### Note
    /// If the header already exists, its value will be overwritten.
    pub fn set_header(
        &mut self,
        name: impl Into<String>,
        value: impl Into<String>,
    ) -> Result<(), RequestError> {
        let header_name = name.into();
        let header_value = value.into();

        let header_name = header_name
            .parse::<HeaderName>()
            .map_err(|_| RequestError::InvalidHeaderName(header_name.clone()))?;

        let header_value = HeaderValue::from_str(&header_value)
            .map_err(|_| RequestError::InvalidHeaderValue(header_value.clone()))?;

        self.headers.insert(header_name, header_value);

        Ok(())
    }

    /// Retrieves and parses a route parameter by name.
    ///
    /// This method extracts URL parameters (path parameters) from the request
    /// and converts them to the specified type. The parameter must implement
    /// the `FromStr` trait for conversion.
    ///
    /// ### Type Parameters
    ///
    /// * `T` - The type to convert the parameter to (must implement `FromStr`)
    ///
    /// ### Arguments
    ///
    /// * `key` - The name of the route parameter to extract
    ///
    /// ### Returns
    ///
    /// Returns `Ok(T)` with the parsed value if the parameter exists and can be
    /// converted, or `Err(RequestError)` if the parameter is missing or invalid.
    ///
    /// ### Errors
    ///
    /// This function will return an error if:
    /// - The parameter is not found in the request
    /// - The parameter value cannot be parsed to type `T`
    ///
    /// ### Example
    ///
    /// ```rust,ignore
    /// use sword::prelude::*;
    ///
    /// ... asuming you have a controller struct ...
    ///
    /// #[get("/users/{id}/posts/{post_id}")]
    /// async fn get_user_post(&self, req: Request) -> HttpResult {
    ///     let user_id: u32 = req.param("id")?;
    ///     let post_id: u64 = req.param("post_id")?;
    ///
    ///     let message = format!("User ID: {}, Post ID: {}", user_id, post_id);
    ///     
    ///     Ok(JsonResponse::Ok().message(message))
    /// }
    /// ```
    pub fn param<T: FromStr>(&self, key: &str) -> Result<T, RequestError> {
        if let Some(value) = self.params.get(key) {
            let Ok(param) = value.parse::<T>() else {
                let message = format!("Invalid parameter format for '{key}'");
                let details = "Failed to deserialize parameter to the required type";

                return Err(RequestError::parse_error(message, details));
            };

            return Ok(param);
        }

        let message = "Parameter not found";
        let details = format!("Parameter '{key}' not found in request parameters");

        Err(RequestError::parse_error(message, details))
    }

    pub const fn params(&self) -> &HashMap<String, String> {
        &self.params
    }

    /// Deserializes the request body from JSON to a specific type.
    ///
    /// This method reads the request body and attempts to parse it as JSON,
    /// deserializing it to the specified type. The body is consumed during
    /// this operation.
    ///
    /// ### Type Parameters
    ///
    /// * `T` - The type to deserialize the JSON body to (must implement `DeserializeOwned`)
    ///
    /// ### Returns
    ///
    /// Returns `Ok(T)` with the deserialized instance if the JSON is valid,
    /// or `Err(RequestError)` if the body is empty or invalid JSON.
    ///
    /// ### Errors
    ///
    /// This function will return an error if:
    /// - The request body is empty
    /// - The body contains invalid JSON
    /// - The JSON structure doesn't match the target type `T`
    ///
    /// ### Example
    ///
    /// ```rust,ignore
    /// use sword::prelude::*;
    /// use serde::Deserialize;
    ///
    /// #[derive(Deserialize)]
    /// struct CreateUserRequest {
    ///     name: String,
    ///     email: String,
    ///     age: u32,
    /// }
    ///
    /// ... asuming you have a controller struct ...
    ///
    /// #[post("/users")]
    /// async fn create_user(&self, req: Request) -> HttpResult {
    ///     let user_data: CreateUserRequest = req.body()?;
    ///     
    ///     // Process user creation...
    ///     
    ///     Ok(JsonResponse::Created().message("User created"))
    /// }
    /// ```
    pub fn body<T: DeserializeOwned>(&self) -> Result<T, RequestError> {
        if self.body_bytes.is_empty() {
            return Err(RequestError::BodyIsEmpty);
        }

        if !self.is_content_type_json() {
            return Err(RequestError::unsupported_media_type(
                "Expected Content-Type to be application/json",
            ));
        }

        serde_json::from_slice(&self.body_bytes).map_err(|e| {
            RequestError::deserialization_error(
                "Invalid request body",
                "Failed to deserialize request body to the required type.".into(),
                e.into(),
            )
        })
    }

    /// Deserializes query parameters from the URL query string to a specific type.
    ///
    /// This method parses the query string portion of the URL and deserializes
    /// it to the specified type. Since query parameters are optional in HTTP,
    /// this method returns `Option<T>` where `None` indicates no query parameters
    /// were present.
    ///
    /// ### Type Parameters
    ///
    /// * `T` - The type to deserialize the query parameters to (must implement `DeserializeOwned`)
    ///
    /// ### Returns
    ///
    /// Returns:
    /// - `Ok(Some(T))` with the deserialized query parameters if they exist and are valid
    /// - `Ok(None)` if no query parameters are present in the URL
    /// - `Err(RequestError)` if query parameters exist but cannot be deserialized
    ///
    /// ### Errors
    ///
    /// This function will return an error if the query parameters exist but
    /// cannot be parsed or deserialized to the target type.
    ///
    /// ### Example
    ///
    /// ```rust,ignore
    /// use sword::prelude::*;
    /// use serde::Deserialize;
    ///
    /// #[derive(Deserialize, Default)]
    /// struct SearchQuery {
    ///     q: Option<String>,
    ///     page: Option<u32>,
    ///     limit: Option<u32>,
    /// }
    ///
    /// ... asuming you have a controller struct ...
    ///
    /// #[get("/search")]
    /// async fn search(&self, req: Request) -> HttpResult {
    ///     let query: SearchQuery = req.query()?.unwrap_or_default();
    ///     
    ///     let search_term = query.q.unwrap_or("".into());
    ///     let page = query.page.unwrap_or(1);
    ///     let limit = query.limit.unwrap_or(20);
    ///     
    ///     Ok(JsonResponse::Ok().data(format!(
    ///         "Search results for '{search_term}', page {page}, limit {limit}"
    ///     )))
    /// }
    /// ```
    pub fn query<T: DeserializeOwned>(&self) -> Result<Option<T>, RequestError> {
        let query_string = self.uri.query().unwrap_or("");

        if query_string.is_empty() {
            return Ok(None);
        }

        let deserializer = serde_urlencoded::Deserializer::new(
            form_urlencoded::parse(query_string.as_bytes()),
        );

        let parsed: T =
            serde_path_to_error::deserialize(deserializer).map_err(|e| {
                // TODO: Implement tracing for loging the errors
                RequestError::deserialization_error(
                    "Invalid query parameters",
                    "Failed to deserialize query parameters to the required type."
                        .into(),
                    e.into(),
                )
            })?;

        Ok(Some(parsed))
    }

    /// Access the cookies from the request.
    /// This method returns a reference to the `Cookies` instance, a struct that provides
    /// methods to get, set, and remove cookies.
    ///
    /// The documentation for `tower_cookies::Cookies` can be found [here](https://docs.rs/tower-cookies/latest/tower_cookies/struct.Cookies.html)
    /// Also, the other cookie-related types like `Cookie`, `CookieBuilder`, `Expiration`, and `SameSite` can be found in the `tower_cookies` crate.
    ///
    /// ### Usage
    /// ```rust,ignore
    ///
    /// use sword::prelude::*;
    ///
    /// ... asuming controller struct ...
    ///
    /// #[get("/show-cookies")]
    /// async fn show_cookies(&self, req: Request) -> HttpResult {
    ///     let cookies = ctx.cookies()?;
    ///     let session_cookie = cookies.get("session_id");
    ///
    ///     if let Some(cookie) = session_cookie {
    ///         Ok(JsonResponse::Ok().body(format!("Session ID: {}", cookie.value())))
    ///     }
    ///
    ///     Ok(JsonResponse::Ok().body("No session cookie found"))
    /// }
    /// ```
    pub fn cookies(&self) -> Result<&Cookies, JsonResponse> {
        self.extensions.get::<Cookies>().ok_or_else(|| {
            JsonResponse::InternalServerError()
                .message("Can't extract cookies. Is `CookieManagerLayer` enabled?")
        })
    }

    /// Returns the value of `Authorization` header if present.
    pub fn authorization(&self) -> Option<&str> {
        self.header("Authorization")
    }

    /// Returns the value of `User-Agent` header if present.
    pub fn user_agent(&self) -> Option<&str> {
        self.header("User-Agent")
    }

    /// Returns the client's IP address from `X-Forwarded-For` header if present.
    pub fn ip(&self) -> Option<&str> {
        self.header("X-Forwarded-For")
    }

    /// Returns a list of IP addresses from `X-Forwarded-For` header if present.
    pub fn ips(&self) -> Option<Vec<&str>> {
        self.header("X-Forwarded-For")
            .map(|ips| ips.split(',').map(|s| s.trim()).collect())
    }

    /// Returns the protocol used in the request from `X-Forwarded-Proto` header if present.
    pub fn protocol(&self) -> &str {
        self.header("X-Forwarded-Proto").unwrap_or("http")
    }

    /// Returns the content length of the request if present.
    pub fn content_length(&self) -> Option<u64> {
        self.header("Content-Length")
            .and_then(|value| value.parse::<u64>().ok())
    }

    /// Returns the unique request ID from the `RequestId` extension if present.
    /// If not present, returns "unknown".
    ///
    /// `RequestId` is added automatically by the `RequestIdLayer` middleware.
    /// If "unknown" is returned, it indicates that the middleware was not applied.
    pub fn id(&self) -> String {
        if let Some(id) = self.extensions.get::<RequestId>() {
            return id.header_value().to_str().unwrap_or_default().to_string();
        }

        "unknown".to_string()
    }

    /// Returns the value of `Content-Type` header if present.
    pub fn content_type(&self) -> Option<&str> {
        self.header("Content-Type")
    }

    #[cfg(feature = "multipart")]
    /// Extracts multipart form data from the request.
    ///
    /// ### Errors
    /// Returns `RequestError::ParseError` if the multipart form data cannot be parsed.
    ///
    /// ### Example
    /// ```rust,ignore
    /// use sword::prelude::*;
    ///
    /// ... asuming a controller struct ...
    ///
    /// #[post("/upload")]
    /// async fn upload(&self, req: Request) -> HttpResult {
    ///     let mut multipart = req.multipart().await?;
    ///     let mut field_names = Vec::new();
    ///
    ///     // Process each field in the multipart form data
    ///     // And ensure to handle errors appropriately
    ///     while let Some(field) = multipart.next_field().await.unwrap() {
    ///         field_names.push(field.name().unwrap_or("Uknown").to_string());
    ///     }
    ///
    ///     Ok(JsonResponse::Ok().data(field_names))
    /// }
    /// ```
    pub async fn multipart(
        self,
    ) -> Result<axum::extract::multipart::Multipart, RequestError> {
        use axum::extract::FromRequest;
        Ok(axum::extract::Multipart::from_request(self.try_into()?, &()).await?)
    }

    pub(crate) fn is_content_type_json(&self) -> bool {
        let Some(content_type) = self.content_type() else {
            return false;
        };

        let Ok(mime) = content_type.parse::<mime::Mime>() else {
            return false;
        };

        mime.type_() == "application"
            && (mime.subtype() == "json"
                || mime.suffix().is_some_and(|name| name == "json"))
    }

    #[doc(hidden)]
    pub fn clear_next(&mut self) {
        self.next = None;
    }

    #[doc(hidden)]
    pub fn set_next(&mut self, next: Next) {
        self.next = Some(next);
    }

    /// Runs the next middleware or handler in the chain.
    ///
    /// This method must be used only in middleware implementations to
    /// pass control to the next middleware or the final request handler.
    pub async fn next(mut self) -> HttpInterceptorResult {
        let Some(next) = self.next.take() else {
            eprintln!(
                "Error: Attempted to call `next()` on Request in a context that is not a `OnRequest` `Interceptor`"
            );
            return Err(JsonResponse::InternalServerError());
        };

        Ok(next.run(self.try_into()?).await)
    }
}
