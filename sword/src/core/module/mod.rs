mod gateway;

use crate::core::{Config, DependencyContainer};

pub use gateway::*;

/// A trait for defining modules in the application.
///
/// `Module` represents a cohesive unit of functionality that can be
/// plugged into the application. Modules can register gateways,
/// components, and providers to extend the application's capabilities.
///
/// # Methods
///
/// - `register_gateways()`: Register entry points into the application (REST APIs, WebSockets, etc.).
///   Use `GatewayRegistry::register::<YourController>()` to register controllers as REST gateways.
/// - `register_components()`: Register component structs marked with `#[injectable]` for dependency injection.
/// - `register_providers()`: Register provider structs marked with `#[injectable(provider)]` asynchronously.
///
/// # Example
///
/// ```rust,ignore
/// use sword::prelude::*;
///
/// pub struct MyModule;
///
/// impl Module for MyModule {
///     fn register_gateways(gateways: &GatewayRegistry) {
///         gateways.register::<MyController>();  // Register REST gateway
///     }
///
///     fn register_components(container: &DependencyContainer) {
///         container.register_component::<MyService>();
///     }
///
///     async fn register_providers(config: &Config, container: &DependencyContainer) {
///         container.register_provider(MyProvider::new().await);
///     }
/// }
/// ```
#[allow(async_fn_in_trait)]
pub trait Module {
    /// Register gateways provided by the module.
    /// A `Gateway` is a way to represent entry points into the application,
    /// such as REST APIs, WebSocket connections, or gRPC services.
    fn register_gateways(_: &GatewayRegistry) {}

    /// Register component structs marked with `#[injectable]`
    fn register_components(_: &DependencyContainer) {}

    /// Register provider structs marked with `#[injectable(provider)]`
    async fn register_providers(_: &Config, _: &DependencyContainer) {}
}
