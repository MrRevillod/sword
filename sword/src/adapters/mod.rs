pub mod rest;

#[cfg(feature = "adapter-socketio")]
pub mod socketio {
    mod adapter;
    mod error;
    mod extract;
    mod interceptor;

    pub use adapter::*;
    pub use error::*;
    pub use extract::*;
    pub use interceptor::*;
}

use parking_lot::{RawRwLock, RwLock, lock_api::RwLockReadGuard};
use std::{any::TypeId, collections::HashMap};
use sword_core::HasDeps;

/// Represents the different kinds of adapters that can be registered.
/// Each variant may hold specific builder functions.
///
/// - Http: The base for RESTful APIs, Multipart data handling, Axum Router with state.
/// - SocketIo: A socketio layer based adapter, Axum Router with state.
#[derive(Debug, PartialEq, Eq, Hash)]
pub enum AdapterKind {
    Http,
    SocketIo,
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
pub trait Adapter: HasDeps {
    fn kind() -> AdapterKind;
    fn type_id() -> TypeId;
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
    adapters: RwLock<HashMap<AdapterKind, Vec<TypeId>>>,
}

impl AdapterRegistry {
    pub(crate) fn new() -> Self {
        Self {
            adapters: RwLock::new(HashMap::new()),
        }
    }

    /// Registers an adapter of type `A` by calling its `kind()` method
    /// and storing the resulting `AdapterKind` in the registry.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// adapters.register::<MyController>();
    /// ```
    pub fn register<A: Adapter>(&self) {
        let mut adapter_registry_vec = self
            .adapters
            .read()
            .get(&A::kind())
            .cloned()
            .unwrap_or_default();

        adapter_registry_vec.push(A::type_id());

        self.adapters
            .write()
            .insert(A::kind(), adapter_registry_vec);
    }

    pub fn read(
        &self,
    ) -> RwLockReadGuard<'_, RawRwLock, HashMap<AdapterKind, Vec<TypeId>>> {
        self.adapters.read()
    }
}

impl Default for AdapterRegistry {
    fn default() -> Self {
        Self::new()
    }
}
