use super::super::ControllerSpec;

pub use sword_macros::{controller, delete, get, patch, post, put};

/// Trait for controllers with automatic dependency injection and interceptors support.
///
/// Controllers in Sword group related route handlers together and can declare
/// dependencies that will be automatically resolved and injected.
///
/// Controllers automatically implement the `ControllerSpec` trait, making them registrable
/// as HTTP controllers within modules via `ControllerRegistry::register::<YourController>()`.
///
/// Use the `#[controller]` macro to automatically implement this trait.
///
/// # Example
///
/// ```rust,ignore
/// use sword::prelude::*;
///
/// #[controller(kind = Controller::Web, path = "/api/users")]
/// struct UserController {
///     service: Arc<UserService>,
/// }
///
/// impl UserController {
///     #[get("/")]
///     async fn list(&self) -> JsonResponse {
///         // Handler logic
///     }
/// }
/// ```
pub trait WebController: ControllerSpec {
    fn base_path() -> &'static str;
}
