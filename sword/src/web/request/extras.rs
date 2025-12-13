use crate::web::Request;
use sword_layers::prelude::RequestId;

impl Request {
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
}
