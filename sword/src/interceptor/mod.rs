use std::sync::Arc;
use sword_core::{Build, DependencyInjectionError, State};

/// Base trait for all interceptors in Sword.
/// Implement this trait to create interceptors that can be automatically
/// registered and built via the dependency injection system.
///
/// This means that the interceptor can have dependencies injected into it,
/// and also be stored one time and reused  throughout the application lifecycle.
pub trait Interceptor: Build {
    fn register(state: &State) -> Result<(), DependencyInjectionError> {
        state.insert(Arc::new(Self::build(state)?));
        Ok(())
    }
}

/// Internal registrar for compile-time interceptor auto-discovery.
///
/// This struct holds a function pointer that can build and register an interceptor
/// instance into the application State. It's used in conjunction with the `inventory`
/// crate to enable automatic interceptor registration at compile time.
///
/// Each interceptor decorated with `#[middleware]` submits a `InterceptorRegistrar`
/// instance to a global collection that is iterated during application initialization.
pub struct InterceptorRegistrar {
    pub register_fn: fn(&State) -> Result<(), DependencyInjectionError>,
}

impl InterceptorRegistrar {
    pub const fn new<I: Interceptor>() -> Self {
        Self {
            register_fn: I::register,
        }
    }
}

inventory::collect!(InterceptorRegistrar);
