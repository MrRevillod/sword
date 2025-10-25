use sword::prelude::*;

#[middleware]
pub struct SetCookieMw;

impl OnRequest for SetCookieMw {
    async fn on_request(&self, mut req: Request, next: Next) -> MiddlewareResult {
        let cookies = req.cookies_mut()?;

        let cookie = CookieBuilder::new("session_id", "abc123")
            .path("/")
            .http_only(true)
            .same_site(SameSite::Lax)
            .build();

        cookies.add(cookie);

        req.run(next).await
    }
}
