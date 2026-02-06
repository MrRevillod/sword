mod adapter;
mod interceptor;
mod request;
mod response;

pub use adapter::*;
pub use interceptor::*;
pub use request::*;
pub use response::*;

pub mod cookies {
    pub use sword_layers::cookies::{
        Cookies, Key as CookiesKey, PrivateCookies, SignedCookies,
        cookie::{
            Cookie, CookieBuilder, Expiration as CookiesExpiration,
            KeyError as CookieKeyError, ParseError as CookieParseError, SameSite,
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

use axum::{Router, routing::MethodRouter};
use std::any::TypeId;
use sword_core::State;

#[derive(Clone)]
pub struct ControllerMeta {
    pub controller_path: &'static str,
    pub apply_top_level_interceptors:
        fn(router: Router<State>, state: State) -> Router<State>,
}

#[derive(Clone)]
pub struct RouteRegistrar {
    /// TypeId of the controller for filtering during registration
    pub controller_id: TypeId,

    /// Base path of the controller (e.g., "/api/users")
    pub controller_path: &'static str,

    /// Path of this specific route (e.g., "/:id")
    pub path: &'static str,

    /// Function that builds the MethodRouter for this route
    /// The closure constructs the controller from state and calls the specific __sword_route_* method
    pub handler: fn(State) -> MethodRouter<State>,

    pub apply_top_level_interceptors:
        fn(router: Router<State>, state: State) -> Router<State>,
}

impl From<&RouteRegistrar> for ControllerMeta {
    fn from(registrar: &RouteRegistrar) -> Self {
        ControllerMeta {
            controller_path: registrar.controller_path,
            apply_top_level_interceptors: registrar.apply_top_level_interceptors,
        }
    }
}

inventory::collect!(RouteRegistrar);
