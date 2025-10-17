mod builtin;

use std::any::Any;

#[cfg(feature = "helmet")]
pub use builtin::helmet;

pub(crate) use builtin::content_type::ContentTypeCheck;
pub(crate) use builtin::prettifier::ResponsePrettifier;

pub use axum::middleware::Next;
pub use sword_macros::{middleware, uses};

use crate::{core::State, errors::DependencyInjectionError};

pub trait Middleware: Any + Send + Sync + 'static {
    fn build(state: &State) -> Result<Self, DependencyInjectionError>
    where
        Self: Sized;
}

pub struct MiddlewareRegistrar {
    pub register: fn(&State) -> Result<(), DependencyInjectionError>,
}

impl MiddlewareRegistrar {
    pub const fn new<M>() -> Self
    where
        M: Middleware + Clone + 'static + Send + Sync,
    {
        fn register_middleware<M: Middleware + Clone + 'static + Send + Sync>(
            state: &State,
        ) -> Result<(), DependencyInjectionError> {
            let mw = M::build(state)?;
            state.insert(mw).map_err(|source| {
                DependencyInjectionError::StateError {
                    type_name: std::any::type_name::<M>().to_string(),
                    source,
                }
            })?;

            Ok(())
        }

        Self {
            register: register_middleware::<M>,
        }
    }
}

inventory::collect!(MiddlewareRegistrar);

/// A macro to simplify the next middleware call in the middleware chain.
///
/// It takes the current context and the next middleware in the chain,
/// and returns a `Result` with the response of the next middleware.
///
/// This macro is used to avoid boilerplate code in middleware implementations.
/// It is used in the `handle` method of the `Middleware` trait.
///
/// # Example usage:
/// ```rust,ignore
/// use sword::prelude::*;
///
/// struct MyMiddleware;
///
/// impl Middleware for MyMiddleware {
///     async fn handle(ctx: Context, next: Next) -> MiddlewareResult {
///         next!(ctx, next)
///     }
/// }
#[macro_export]
macro_rules! next {
    ($ctx:expr, $next:expr) => {
        Ok($next.run($ctx.try_into()?).await)
    };
}
