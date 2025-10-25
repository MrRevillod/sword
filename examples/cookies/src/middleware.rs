use sword::prelude::*;

#[middleware]
pub struct SetCookieMw;

impl OnRequest for SetCookieMw {
    async fn on_request(&self, req: Request) -> MiddlewareResult {
        let cookies = req.cookies()?;

        let cookie = CookieBuilder::new("session_id", "abc123")
            .path("/")
            .http_only(true)
            .same_site(SameSite::Lax)
            .build();

        cookies.add(cookie);

        req.next().await
    }
}
