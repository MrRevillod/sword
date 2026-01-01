pub mod rest;

#[cfg(feature = "adapter-socketio")]
pub mod socketio {
    mod adapter;
    mod extract;
    mod interceptor;

    pub use adapter::*;
    pub use extract::*;
    pub use interceptor::*;
}

use axum::Router;
use parking_lot::RwLock;
use sword_core::State;

/// Represents the different kinds of adapters that can be registered.
/// Each variant may hold specific builder functions.
///
/// - Rest: The base for RESTful APIs, Axum Router with state.
/// - WebSocket: A socketio layer based adapter, Axum Router with state.
/// - Grpc: Not implemented yet.
pub enum AdapterKind {
    Rest(Box<dyn Fn(State) -> Router>),
    SocketIo(Box<dyn Fn(&State)>),
    Grpc,
}

/// A trait for defining adapters in the application.
///
/// Adapters represent different entry points into your application. Controllers
/// automatically implement this trait, allowing them to be registered as REST adapters
/// within modules.
///
/// # Example
///
/// ```rust,ignore
/// use sword::prelude::*;
///
/// #[controller("/api/items")]
/// struct ItemsController { /* ... */ }
///
/// // The macro automatically implements Adapter for ItemsController
/// // In your module:
/// fn register_adapters(adapters: &AdapterRegistry) {
///     adapters.register::<ItemsController>();
/// }
/// ```
pub trait Adapter {
    fn kind() -> AdapterKind;
}

/// Registry for managing and storing different adapter kinds.
///
/// `AdapterRegistry` is used within modules to register adapters that define how requests
/// enter the application.
///
/// # Example
///
/// ```rust,ignore
/// use sword::prelude::*;
///
/// struct MyModule;
///
/// impl Module for MyModule {
///     fn register_adapters(adapters: &AdapterRegistry) {
///         adapters.register::<UserController>();
///         adapters.register::<ProductController>();
///     }
/// }
/// ```
pub struct AdapterRegistry {
    adapters: RwLock<Vec<AdapterKind>>,
}

impl AdapterRegistry {
    pub fn new() -> Self {
        Self {
            adapters: RwLock::new(Vec::new()),
        }
    }

    /// Registers an adapter of type `G` by calling its `kind()` method
    /// and storing the resulting `AdapterKind` in the registry.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// adapters.register::<MyController>();
    /// ```
    pub fn register<A: Adapter>(&self) {
        self.adapters.write().push(A::kind());
    }

    pub fn inner(&self) -> &RwLock<Vec<AdapterKind>> {
        &self.adapters
    }

    pub fn clear(&self) {
        self.adapters.write().clear();
    }
}

impl Default for AdapterRegistry {
    fn default() -> Self {
        Self::new()
    }
}
