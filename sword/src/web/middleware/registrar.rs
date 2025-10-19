use std::{any::type_name, sync::Arc};

use crate::{
    core::{DependencyInjectionError, State},
    web::Middleware,
};

pub struct MiddlewareRegistrar {
    pub register_fn: fn(&State) -> Result<(), DependencyInjectionError>,
}

impl MiddlewareRegistrar {
    pub const fn new<M: Middleware>() -> Self {
        fn register_fn<M: Middleware>(
            state: &State,
        ) -> Result<(), DependencyInjectionError> {
            state.insert(Arc::new(M::build(state)?)).map_err(|e| {
                DependencyInjectionError::state_error(type_name::<M>(), e)
            })?;

            Ok(())
        }

        Self {
            register_fn: register_fn::<M>,
        }
    }
}

inventory::collect!(MiddlewareRegistrar);
