use std::sync::Arc;
use sword_core::{Build, State};

/// Base trait for all interceptors in Sword.
/// Implement this trait to create interceptors that can be automatically
/// registered and built via the dependency injection system.
///
/// This means that the interceptor can have dependencies injected into it,
/// and also be stored one time and reused  throughout the application lifecycle.
pub trait Interceptor: Build {
    fn register(state: &State) -> () {
        let interceptor = Self::build(state).map_err(|err| {
            panic!("\n[!] Failed to build Interceptor\n\n{err}\n");
        });

        state.insert(Arc::new(interceptor));
    }
}

pub struct InterceptorRegistrar {
    pub register: fn(&State) -> (),
}

impl InterceptorRegistrar {
    pub const fn new<I: Interceptor>() -> Self {
        Self {
            register: I::register,
        }
    }
}

inventory::collect!(InterceptorRegistrar);
