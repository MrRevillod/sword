mod controller;
mod interceptor;
mod request;
mod response;

pub use controller::*;
pub use interceptor::*;
pub use request::*;
pub use response::*;

#[cfg(feature = "web-controllers")]
pub mod cookies {
    pub use sword_layers::cookies::{
        Cookies, Key as CookiesKey, PrivateCookies, SignedCookies,
        cookie::{
            Cookie, CookieBuilder, Expiration as CookiesExpiration, KeyError as CookieKeyError,
            ParseError as CookieParseError, SameSite,
        },
    };
}

#[cfg(feature = "multipart")]
pub mod multipart {
    pub use axum::extract::multipart::{
        Field, InvalidBoundary, Multipart, MultipartError, MultipartRejection,
    };
    pub use bytes::*;
}

use axum::routing::MethodRouter;
use std::any::TypeId;
use sword_core::State;

#[derive(Clone)]
pub struct WebControllerRegistrar {
    pub controller_id: TypeId,
    pub controller_path: &'static str,
}

#[derive(Clone)]
pub struct RouteRegistrar {
    /// TypeId of the controller for filtering during registration
    pub controller_id: TypeId,

    /// Path of this specific route (e.g., "/{id}")
    pub path: &'static str,

    /// Function that builds the MethodRouter for this route
    /// The closure constructs the controller from state and calls the specific __sword_route_* method
    pub handler: fn(State) -> MethodRouter<State>,
}

inventory::collect!(WebControllerRegistrar);
inventory::collect!(RouteRegistrar);
