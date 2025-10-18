use crate::web::*;

const APPLICATION_JSON: &str = "application/json";
const MULTIPART_FORM_DATA: &str = "multipart/form-data";

pub(crate) struct ContentTypeCheck;

impl ContentTypeCheck {
    pub async fn layer(ctx: Context, next: Next) -> MiddlewareResult {
        let content_type = ctx.header("Content-Type").unwrap_or_default();

        if !ctx.has_body() {
            return next!(ctx, next);
        }

        if content_type != APPLICATION_JSON
            && !content_type.contains(MULTIPART_FORM_DATA)
        {
            return Err(HttpResponse::UnsupportedMediaType().message(
                "Only application/json and multipart/form-data content types are supported.",
            ));
        }

        next!(ctx, next)
    }
}
