mod container;
mod error;

use std::{any::Any, sync::Arc};

use crate::core::{Build, HasDeps, State};

pub use container::DependencyContainer;
pub use error::DependencyInjectionError;
pub use sword_macros::injectable;

/// Pointer to dyn Any element. It retrieves dynamic capabilites
/// to the dependency container. Basically represents Any element.
type Dependency = Arc<dyn Any + Send + Sync>;

/// A function that builds a dependency from application State
type DependencyBuilder =
    Box<dyn Fn(&State) -> Result<Dependency, DependencyInjectionError>>;

/// Trait for injectable components that can be automatically constructed
/// by the dependency container with automatic dependency resolution.
///
/// Components are services or dependencies that need to be built from other
/// components in the State. The framework automatically resolves the dependency
/// graph using topological sorting to ensure all dependencies are available
/// before constructing a component.
///
/// Use the `#[injectable]` macro to automatically implement this trait.
///
/// # Example
///
/// ```rust,ignore
/// use sword::prelude::*;
///
/// #[injectable]
/// struct UserService {
///     db: Arc<Database>,
///     cache: Arc<CacheService>,
/// }
///
/// impl UserService {
///     fn get_user(&self, id: u64) -> User {
///         // Service logic
///     }
/// }
/// ```
pub trait Component: HasDeps<Error = DependencyInjectionError> {}

/// Marker trait for pre-instantiated dependencies (providers).
///
/// Providers are dependencies that cannot be auto-constructed from the State
/// (e.g., database connections, external API clients) but need to be available
/// for injection into other components.
pub trait Provider: Build<Error = DependencyInjectionError> {}
