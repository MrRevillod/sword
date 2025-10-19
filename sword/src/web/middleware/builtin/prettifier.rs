use axum::http::StatusCode;
use axum_responses::http::HttpResponse;

use crate::web::*;

pub struct ResponsePrettifier;

impl ResponsePrettifier {
    pub async fn layer(req: Request, next: Next) -> MiddlewareResult {
        let response = next.run(req.try_into()?).await;

        if response.status() == StatusCode::REQUEST_TIMEOUT {
            return Err(HttpResponse::RequestTimeout());
        }

        Ok(response)
    }
}
