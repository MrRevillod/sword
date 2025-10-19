use crate::core::{DependencyInjectionError, HasDeps, State};
use axum::Router as AxumRouter;

pub use sword_macros::{controller, delete, get, patch, post, put, routes};

pub trait Controller: ControllerBuilder {
    fn router(state: State) -> AxumRouter;
}

/// Trait for controllers with automatic dependency injection and middleware support.
///
/// Controllers in Sword group related route handlers together and can declare
/// dependencies that will be automatically resolved and injected.
///
/// Use the `#[controller]` macro to automatically implement this trait.
///
/// # Example
///
/// ```rust,ignore
/// use sword::prelude::*;
///
/// #[controller("/api/users")]
/// struct UserController {
///     service: Arc<UserService>,
/// }
///
/// #[routes]
/// impl UserController {
///     #[get("/")]
///     async fn list(&self) -> HttpResponse {
///         // Handler logic
///     }
/// }
/// ```
pub trait ControllerBuilder: HasDeps<Error = DependencyInjectionError> {
    fn base_path() -> &'static str;
    fn apply_middlewares(router: AxumRouter, state: State) -> AxumRouter;
}
