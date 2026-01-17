mod adapter;
mod interceptor;
mod request;
mod response;

pub use adapter::*;
pub use interceptor::*;
pub use request::*;
pub use response::*;

pub mod extractors {
    pub use super::request::extractors::*;
}

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

use axum::routing::MethodRouter;
use std::any::TypeId;
use sword_core::State;

/// Registry entry for auto-registering individual REST routes.
///
/// Each #[get], #[post], etc. macro registers a route using `inventory::submit!`.
/// Routes are grouped by controller at runtime when building the application router.
///
/// This approach eliminates the macro expansion order problem by deferring
/// all route registration to runtime after all macros have expanded.
///
/// The build_handler creates the MethodRouter directly without needing the controller
/// to be built during app initialization - controllers are built lazily per-request.
pub struct RouteRegistrar {
    /// TypeId of the controller for filtering during registration
    pub controller_type_id: TypeId,

    /// Name of the controller (e.g., "UsersController")
    pub controller_name: &'static str,

    /// Base path of the controller (e.g., "/api/users")
    pub controller_path: &'static str,

    /// Path of this specific route (e.g., "/:id")
    pub route_path: &'static str,

    /// Function that builds the MethodRouter for this route
    /// The closure constructs the controller from state and calls the specific __sword_route_* method
    /// Returns MethodRouter<State> - handlers can use State<S> extractor
    pub build_handler: fn(State) -> MethodRouter<State>,

    /// Function that applies controller-level interceptors to a router
    /// Takes Router<State> and State, returns Router<State>
    pub apply_interceptors: fn(axum::Router<State>, State) -> axum::Router<State>,
}

impl RouteRegistrar {
    pub const fn new(
        controller_type_id: TypeId,
        controller_name: &'static str,
        controller_path: &'static str,
        route_path: &'static str,
        build_handler: fn(State) -> MethodRouter<State>,
        apply_interceptors: fn(axum::Router<State>, State) -> axum::Router<State>,
    ) -> Self {
        Self {
            controller_type_id,
            controller_name,
            controller_path,
            route_path,
            build_handler,
            apply_interceptors,
        }
    }
}

inventory::collect!(RouteRegistrar);
