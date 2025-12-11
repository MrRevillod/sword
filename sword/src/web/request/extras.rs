use crate::web::Request;

impl Request {
    pub fn authorization(&self) -> Option<&str> {
        self.header("Authorization")
    }

    pub fn user_agent(&self) -> Option<&str> {
        self.header("User-Agent")
    }

    pub fn ip(&self) -> Option<&str> {
        self.header("X-Forwarded-For")
    }

    pub fn ips(&self) -> Option<Vec<&str>> {
        self.header("X-Forwarded-For")
            .map(|ips| ips.split(',').map(|s| s.trim()).collect())
    }

    pub fn protocol(&self) -> &str {
        self.header("X-Forwarded-Proto").unwrap_or("http")
    }

    pub fn content_length(&self) -> Option<u64> {
        self.header("Content-Length")
            .and_then(|value| value.parse::<u64>().ok())
    }

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
