use crate::web::*;

pub struct ContentTypeCheck;

impl ContentTypeCheck {
    pub async fn layer(mut req: Request, next: Next) -> MiddlewareResult {
        req.set_next(next);

        let content_type = req.header("Content-Type").unwrap_or_default();

        if !req.has_body() {
            return req.next().await;
        }

        if content_type != "application/json"
            && !content_type.contains("multipart/form-data")
        {
            return Err(HttpResponse::UnsupportedMediaType().message(
                "Only application/json and multipart/form-data content types are supported.",
            ));
        }

        req.next().await
    }
}
