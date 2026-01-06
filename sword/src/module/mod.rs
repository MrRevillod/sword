use crate::adapters::AdapterRegistry;
use sword_core::{ComponentRegistry, Config, ProviderRegistry};

/// A trait for defining modules in the application.
///
/// `Module` represents a cohesive unit of functionality that can be
/// plugged into the application. Modules can register adapters,
/// components, and providers to extend the application's capabilities.
///
/// # Example
///
/// ```rust,ignore
/// use sword::prelude::*;
///
/// pub struct MyModule;
///
/// impl Module for MyModule {
///     fn register_adapters(adapters: &AdapterRegistry) {
///         adapters.register::<MyController>();  // Register REST adapter
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
    /// Register adapters provided by the module.
    /// A `Adapter` is a way to represent entry points into the application,
    /// such as REST APIs, Socket.IO Handlers, or gRPC services.
    fn register_adapters(_: &AdapterRegistry) {}

    /// Register component structs marked with `#[injectable]`
    fn register_components(_: &ComponentRegistry) {}

    /// Register provider structs marked with `#[injectable(provider)]`
    async fn register_providers(_: &Config, _: &ProviderRegistry) {}
}
