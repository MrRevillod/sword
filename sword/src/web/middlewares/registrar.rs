use std::sync::Arc;

use crate::{
    core::{DependencyInjectionError, State},
    web::Middleware,
};

/// Internal registrar for compile-time middleware auto-discovery.
///
/// This struct holds a function pointer that can build and register a middleware
/// instance into the application State. It's used in conjunction with the `inventory`
/// crate to enable automatic middleware registration at compile time.
///
/// Each middleware decorated with `#[middleware]` submits a `MiddlewareRegistrar`
/// instance to a global collection that is iterated during application initialization.
pub struct MiddlewareRegistrar {
    pub register_fn: fn(&State) -> Result<(), DependencyInjectionError>,
}

impl MiddlewareRegistrar {
    /// Creates a new registrar for a specific middleware type.
    ///
    /// This method is `const` to allow compile-time evaluation when used with
    /// `inventory::submit!`. The returned registrar contains a type-erased
    /// function that knows how to build and register the middleware.
    pub const fn new<M: Middleware>() -> Self {
        fn register_fn<M>(state: &State) -> Result<(), DependencyInjectionError>
        where
            M: Middleware,
        {
            state.insert(Arc::new(M::build(state)?));

            Ok(())
        }

        Self {
            register_fn: register_fn::<M>,
        }
    }
}

inventory::collect!(MiddlewareRegistrar);
