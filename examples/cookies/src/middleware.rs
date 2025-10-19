use sword::prelude::*;

#[middleware]
pub struct SetCookieMw;

impl SetCookieMw {
    #[on_request]
    async fn handle(&self, mut req: Request, next: Next) -> MiddlewareResult {
        let cookies = req.cookies_mut()?;

        let cookie = CookieBuilder::new("session_id", "abc123")
            .path("/")
            .http_only(true)
            .same_site(SameSite::Lax)
            .build();

        cookies.add(cookie);

        next!(req, next)
    }
}
