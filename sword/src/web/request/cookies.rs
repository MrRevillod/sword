use crate::web::{HttpResponse, Request};

pub use tower_cookies::{
    Cookies, Key, PrivateCookies, SignedCookies,
    cookie::{
        Cookie, CookieBuilder, Expiration, KeyError as CookieKeyError,
        ParseError as CookieParseError, SameSite,
    },
};

impl Request {
    /// Access the cookies from the request.
    /// This method returns a reference to the `Cookies` instance, a struct that provides
    /// methods to get, set, and remove cookies.
    ///
    /// To get a mutable reference to the cookies, use the `cookies_mut` method.
    ///
    /// The documentation for `tower_cookies::Cookies` can be found [here](https://docs.rs/tower-cookies/latest/tower_cookies/struct.Cookies.html)
    /// Also, the other cookie-related types like `Cookie`, `CookieBuilder`, `Expiration`, and `SameSite` can be found in the `tower_cookies` crate.
    ///
    /// ### Usage
    /// ```rust,ignore
    ///
    /// use sword::prelude::*;
    ///
    /// ... asuming you have a controller struct ...
    ///
    /// #[get("/show-cookies")]
    /// async fn show_cookies(&self, req: Request) -> HttpResult {
    ///     let cookies = ctx.cookies()?;
    ///     let session_cookie = cookies.get("session_id");
    ///
    ///     if let Some(cookie) = session_cookie {
    ///         Ok(HttpResponse::Ok().body(format!("Session ID: {}", cookie.value())))
    ///     }
    ///
    ///     Ok(HttpResponse::Ok().body("No session cookie found"))
    /// }
    /// ```
    pub fn cookies(&self) -> Result<&Cookies, HttpResponse> {
        self.extensions.get::<Cookies>().ok_or_else(|| {
            HttpResponse::InternalServerError()
                .message("Can't extract cookies. Is `CookieManagerLayer` enabled?")
        })
    }
}
