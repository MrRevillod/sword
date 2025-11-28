mod container;
mod error;

use std::{
    any::{Any, TypeId},
    sync::Arc,
};

use crate::core::State;

pub use container::DependencyContainer;
pub use error::DependencyInjectionError;
pub use sword_macros::injectable;

/// Base trait for any component that can be constructed from the application State.
pub trait Build: Clone + Send + Sync + 'static {
    fn build(state: &State) -> Result<Self, DependencyInjectionError>
    where
        Self: Sized;
}

/// Trait for components that have dependencies on other components.
///
/// The `deps()` method returns a list of `TypeId`s of the dependencies
/// required to build the component.
pub trait HasDeps: Build {
    fn deps() -> Vec<TypeId> {
        Vec::new()
    }
}

/// Pointer to dyn Any element. It retrieves dynamic capabilites
/// to the dependency container. Basically represents Any element.
type Dependency = Arc<dyn Any + Send + Sync>;

/// A function that builds a dependency from application State
type DependencyBuilderFn =
    Box<dyn Fn(&State) -> Result<Dependency, DependencyInjectionError>>;

/// Trait for injectable components that can be automatically constructed
/// by the dependency container with automatic dependency resolution.
///
/// Components are services or dependencies that need to be built from other
/// components in the State.
///
/// Use the `#[injectable]` macro to automatically implement this trait.
pub trait Component: HasDeps {}

/// Marker trait for pre-instantiated dependencies (providers).
///
/// Providers are dependencies that cannot be auto-constructed from the State
/// (e.g., database connections, external API clients) but need to be available
/// for injection into other components.
pub trait Provider: Build {}
