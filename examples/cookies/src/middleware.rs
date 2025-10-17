// use sword::prelude::*;

// pub struct SetCookieMw {}

// impl Middleware for SetCookieMw {
//     async fn handle(mut ctx: Context, next: Next) -> MiddlewareResult {
//         let cookies = ctx.cookies_mut()?;

//         let cookie = CookieBuilder::new("session_id", "abc123")
//             .path("/")
//             .http_only(true)
//             .same_site(SameSite::Lax)
//             .build();

//         cookies.add(cookie);

//         next!(ctx, next)
//     }
// }
