mod container;

use std::{
    any::{Any, TypeId},
    sync::Arc,
};

use crate::{core::State, errors::DependencyInjectionError};
pub use container::DependencyContainer;

/// Pointer to dyn Any element. It retrieves dynamic capabilites
/// to the dependency container. Basically represents Any element.
type Dependency = Arc<dyn Any + Send + Sync>;

/// A function that builds a dependency from application State
type DependencyBuilder =
    Box<dyn Fn(&State) -> Result<Dependency, DependencyInjectionError>>;

/// Trait to represent elements that can be injected and built automatically
/// by the dependency container.
///
/// This trait gives two functions that helps to build a dependency and
/// its own dependencies in recursive way.
pub trait Component: Send + Sync + 'static {
    fn build(state: &State) -> Result<Self, DependencyInjectionError>
    where
        Self: Sized;

    fn dependencies() -> Vec<TypeId>;
}

/// Marker trait for types that are manually instantiated and registered as providers.
///
/// Instances are dependencies that cannot be auto-constructed from the State
/// (e.g., database connections, external API clients) but need to be available
/// for injection into other services.
pub trait Provider: Send + Sync + 'static {}
