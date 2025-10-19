use std::any::TypeId;

use super::State;

/// Base trait for any component that can be constructed from the application State.
///
/// This trait unifies the construction of all types of components in the framework:
/// Middlewares, Components, Providers, Controllers, etc. It provides a consistent
/// interface for building instances from the shared application state.
///
/// All components must be thread-safe (`Send + Sync + 'static`) to work in an
/// asynchronous web server environment.
pub trait Build: Send + Sync + 'static {
    type Error: std::error::Error + Send + Sync + 'static;

    /// Constructs an instance of this type from the application State.
    ///
    /// This method should extract any required dependencies from the State
    /// and use them to construct a new instance.
    fn build(state: &State) -> Result<Self, Self::Error>
    where
        Self: Sized;
}

/// Trait for components that need to be cloneable.
pub trait Clonable: Build + Clone {}

/// Trait for components that have dependencies on other components.
///
/// This trait extends `Build` to provide dependency tracking capabilities, allowing
/// the dependency injection system to automatically resolve the dependency graph
/// using topological sorting.
///
/// The `deps()` method returns a list of `TypeId`s representing the types that
/// this component depends on.
pub trait HasDeps: Build {
    fn deps() -> Vec<TypeId> {
        Vec::new()
    }
}

/// Trait for components that support compile-time auto-discovery.
///
/// This trait enables components to be automatically discovered and registered
/// at compile time using the `inventory` crate. It's primarily used internally
/// by the framework for auto-registering middlewares.
///
/// The associated `Registrar` type is responsible for registering and constructing
/// instances of the component during application initialization.
pub trait Discoverable: Build {
    type Registrar: Send + Sync + 'static;
    fn registrar() -> Self::Registrar;
}
