mod middleware;

use middleware::SetCookieMw;
use serde::Deserialize;
use sword::prelude::*;

#[derive(Clone, Deserialize)]
#[config(key = "cookies")]
struct CookiesConfig {
    key: String,
}

#[controller("/cookies")]
struct CookieController {
    cookies_config: CookiesConfig,
}

#[routes]
impl CookieController {
    #[get("/set")]
    async fn set_cookie(&self, mut req: Request) -> HttpResult {
        let cookies = req.cookies_mut()?;

        let cookie = CookieBuilder::new("username", "sword_user")
            .path("/")
            .http_only(true)
            .same_site(SameSite::Lax)
            .build();

        cookies.add(cookie);

        Ok(HttpResponse::Ok())
    }

    #[get("/with_middleware")]
    #[uses(SetCookieMw)]
    async fn with_middleware(&self, mut req: Request) -> HttpResult {
        let cookies = req.cookies_mut()?;

        let session_cookie = cookies.get("session_id").ok_or_else(|| {
            HttpResponse::Unauthorized().message("Session cookie not found")
        })?;

        Ok(HttpResponse::Ok()
            .message(format!("Session ID: {}", session_cookie.value())))
    }

    #[get("/private-counter")]
    async fn private_counter(&self, mut req: Request) -> HttpResult {
        let key = Key::from(self.cookies_config.key.as_bytes());
        let private = req.cookies_mut()?.private(&key);

        let count = private
            .get("visited_private")
            .and_then(|c| c.value().parse::<u32>().ok())
            .unwrap_or(0);

        if count > 10 {
            private.remove(Cookie::new("visited_private", ""));

            Ok(HttpResponse::Ok()
                .message("You've visited more than 10 times, resetting counter."))
        } else {
            private.add(Cookie::new("visited_private", (count + 1).to_string()));

            Ok(HttpResponse::Ok()
                .message(format!("You've been {count} times before")))
        }
    }
}

#[sword::main]
async fn main() {
    let app = Application::builder()
        .with_controller::<CookieController>()
        .build();

    app.run().await;
}
