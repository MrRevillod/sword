use std::{collections::HashMap, str::FromStr};

use axum::http::Method;
use serde::de::DeserializeOwned;

#[cfg(feature = "validator")]
pub mod validator;

#[cfg(feature = "validator")]
pub use validator::ValidatorRequestValidation;

use crate::{errors::RequestError, web::Context};

impl Context {
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
    /// * `key` - The header name to search for (case-insensitive).
    ///
    /// ### Returns
    /// `Some(&str)` with the header value if it exists, `None` if not found.
    pub fn header(&self, key: &str) -> Option<&str> {
        self.headers.get(&key.to_lowercase()).map(String::as_str)
    }

    /// Gets an immutable reference to all request headers.
    ///
    /// ### Returns
    /// A reference to `HashMap<String, String>` containing all request headers
    /// where the key is the header name and the value is its content.
    pub const fn headers(&self) -> &HashMap<String, String> {
        &self.headers
    }

    /// Gets a mutable reference to all request headers.
    ///
    /// ### Returns
    /// A mutable reference to `HashMap<String, String>` that allows modifying
    /// existing headers or adding new headers to the request.
    pub const fn headers_mut(&mut self) -> &mut HashMap<String, String> {
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
    pub fn set_header(&mut self, name: impl Into<String>, value: impl Into<String>) {
        self.headers.insert(name.into(), value.into());
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
    /// // Route: GET /users/{id}/posts/{post_id}
    /// #[get("/users/{id}/posts/{post_id}")]
    /// async fn get_user_post(&self, ctx: Context) -> HttpResult<HttpResponse> {
    ///     let user_id: u32 = ctx.param("id")?;
    ///     let post_id: u64 = ctx.param("post_id")?;
    ///
    ///     let message = format!("User ID: {}, Post ID: {}", user_id, post_id);
    ///     
    ///     Ok(HttpResponse::Ok().message(message))
    /// }
    /// ```
    pub fn param<T: FromStr>(&self, key: &str) -> Result<T, RequestError> {
        if let Some(value) = self.params.get(key) {
            let Ok(param) = value.parse::<T>() else {
                let message = "Invalid parameter type";
                let details = "Failed to parse parameter to the required type";

                return Err(RequestError::ParseError(message, details.into()));
            };

            return Ok(param);
        }

        let message = "Parameter not found";
        let details = format!("Parameter '{key}' not found in request parameters");

        Err(RequestError::ParseError(message, details))
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
    /// #[post("/users")]
    /// async fn create_user(&self, ctx: Context) -> HttpResult<HttpResponse> {
    ///     let user_data: CreateUserRequest = ctx.body()?;
    ///     
    ///     // Process user creation...
    ///     
    ///     Ok(HttpResponse::Created().message("User created"))
    /// }
    /// ```
    pub fn body<T: DeserializeOwned>(&self) -> Result<T, RequestError> {
        if self.body_bytes.is_empty() {
            return Err(RequestError::BodyIsEmpty("Request body is empty"));
        }

        serde_json::from_slice(&self.body_bytes).map_err(|_| {
            let message = "Invalid request body";
            let details = "Failed to parse request body to the required type.";

            RequestError::ParseError(message, details.into())
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
    /// // Route: GET /search?q=rust&page=1&limit=10
    /// #[get("/search")]
    /// async fn search(&self, ctx: Context) -> HttpResult<HttpResponse> {
    ///     let query: SearchQuery = ctx.query()?.unwrap_or_default();
    ///     
    ///     let search_term = query.q.unwrap_or("".into());
    ///     let page = query.page.unwrap_or(1);
    ///     let limit = query.limit.unwrap_or(20);
    ///     
    ///     Ok(HttpResponse::Ok().data(format!(
    ///         "Search results for '{}', page {}, limit {}",
    ///         search_term, page, limit
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
            serde_path_to_error::deserialize(deserializer).map_err(|_| {
                let message = "Invalid query parameters";
                let details =
                    "Failed to parse query parameters to the required type.";
                RequestError::ParseError(message, details.into())
            })?;

        Ok(Some(parsed))
    }

    /// Checks if the request has a non-empty body.
    ///
    /// This is an internal method used by the framework to determine
    /// if the request contains body data. It's primarily used for
    /// internal request processing logic.
    ///
    /// ### Returns
    ///
    /// Returns `true` if the request has a body with content, `false` if empty.
    pub(crate) const fn has_body(&self) -> bool {
        !self.body_bytes.is_empty()
    }
}
