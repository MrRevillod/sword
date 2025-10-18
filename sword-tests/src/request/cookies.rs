use axum_test::TestServer;
use sword::prelude::*;

// struct SetCookieMw {}

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

#[controller("/cookies")]
struct CookieController {}

#[routes]
impl CookieController {
    #[get("/set")]
    async fn set_cookie(&self, mut ctx: Context) -> HttpResult {
        let cookies = ctx.cookies_mut()?;

        let cookie = CookieBuilder::new("username", "sword_user")
            .path("/")
            .http_only(true)
            .same_site(SameSite::Lax)
            .build();

        cookies.add(cookie);

        Ok(HttpResponse::Ok())
    }

    #[get("/with_middleware")]
    // #[middleware(SetCookieMw)]
    async fn with_middleware(&self, mut ctx: Context) -> HttpResult {
        let cookies = ctx.cookies_mut()?;

        let session_cookie = cookies.get("session_id").ok_or(
            HttpResponse::Unauthorized().message("Session cookie not found"),
        )?;

        Ok(HttpResponse::Ok()
            .message(format!("Session ID: {}", session_cookie.value())))
    }
}

#[tokio::test]
async fn test_set_cookie() -> Result<(), Box<dyn std::error::Error>> {
    let app = Application::builder()
        .with_controller::<CookieController>()
        .build();

    let server = TestServer::new(app.router())?;

    let response = server.get("/cookies/set").await;
    assert_eq!(response.status_code(), 200);

    let cookies = response.cookies();

    let username_cookie = cookies
        .iter()
        .find(|cookie| cookie.name() == "username")
        .expect("Cookie 'username' not found");

    assert_eq!(username_cookie.value(), "sword_user");

    assert_eq!(username_cookie.path(), Some("/"));
    assert!(username_cookie.http_only().unwrap_or(false));
    assert_eq!(username_cookie.same_site(), Some(SameSite::Lax));

    Ok(())
}

#[tokio::test]
async fn test_with_middleware() -> Result<(), Box<dyn std::error::Error>> {
    let app = Application::builder()
        .with_controller::<CookieController>()
        .build();

    let server = TestServer::new(app.router())?;

    let response = server.get("/cookies/with_middleware").await;
    assert_eq!(response.status_code(), 200);

    let cookies = response.cookies();

    let session_cookie = cookies
        .iter()
        .find(|cookie| cookie.name() == "session_id")
        .expect("Cookie 'session_id' not found");

    assert_eq!(session_cookie.value(), "abc123");

    assert_eq!(session_cookie.path(), Some("/"));
    assert!(session_cookie.http_only().unwrap_or(false));
    assert_eq!(session_cookie.same_site(), Some(SameSite::Lax));

    Ok(())
}
