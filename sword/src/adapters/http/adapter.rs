use super::super::Adapter;
use axum::Router as AxumRouter;
use sword_core::State;

pub use sword_macros::{controller, delete, get, patch, post, put};

/// Trait for controllers with automatic dependency injection and interceptors support.
///
/// Controllers in Sword group related route handlers together and can declare
/// dependencies that will be automatically resolved and injected.
///
/// Controllers automatically implement the `Adapter` trait, making them registrable
/// as HTTP controllers within modules via `AdapterRegistry::register::<YourController>()`.
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
///     async fn list(&self) -> JsonResponse {
///         // Handler logic
///     }
/// }
/// ```
pub trait HttpController: Adapter {
    fn base_path() -> &'static str;
    fn apply_interceptors(
        router: AxumRouter<State>,
        state: State,
    ) -> AxumRouter<State>;
}
