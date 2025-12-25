use crate::core::{Adapter, HasDeps, State};
use axum::Router as AxumRouter;

pub use sword_macros::{controller, delete, get, patch, post, put, routes};

/// Trait for controllers with automatic dependency injection and middleware support.
///
/// Controllers in Sword group related route handlers together and can declare
/// dependencies that will be automatically resolved and injected.
///
/// Controllers automatically implement the `Adapter` trait, making them registrable
/// as REST adapters within modules via `AdapterRegistry::register::<YourController>()`.
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
pub trait Controller: HasDeps + Adapter {
    fn base_path() -> &'static str;
    fn apply_middlewares(router: AxumRouter, state: State) -> AxumRouter;
}
