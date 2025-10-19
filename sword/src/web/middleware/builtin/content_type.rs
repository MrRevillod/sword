use crate::web::*;

const APPLICATION_JSON: &str = "application/json";
const MULTIPART_FORM_DATA: &str = "multipart/form-data";

pub(crate) struct ContentTypeCheck;

impl ContentTypeCheck {
    pub async fn layer(req: Request, next: Next) -> MiddlewareResult {
        let content_type = req.header("Content-Type").unwrap_or_default();

        if !req.has_body() {
            return next!(req, next);
        }

        if content_type != APPLICATION_JSON
            && !content_type.contains(MULTIPART_FORM_DATA)
        {
            return Err(HttpResponse::UnsupportedMediaType().message(
                "Only application/json and multipart/form-data content types are supported.",
            ));
        }

        next!(req, next)
    }
}
