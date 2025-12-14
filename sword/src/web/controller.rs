use crate::core::{Gateway, HasDeps, State};
use axum::Router as AxumRouter;

pub use sword_macros::{controller, delete, get, patch, post, put, routes};

/// Trait for controllers with automatic dependency injection and middleware support.
///
/// Controllers in Sword group related route handlers together and can declare
/// dependencies that will be automatically resolved and injected.
pub trait Controller: HasDeps + Gateway {
    fn base_path() -> &'static str;
    fn apply_middlewares(router: AxumRouter, state: State) -> AxumRouter;
}
