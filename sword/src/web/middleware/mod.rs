mod builtin;

#[cfg(feature = "helmet")]
pub use builtin::helmet;

pub(crate) use builtin::content_type::ContentTypeCheck;
pub(crate) use builtin::prettifier::ResponsePrettifier;

pub use axum::middleware::Next;
pub use sword_macros::{middleware, use_middleware};

use crate::{core::State, errors::DependencyInjectionError};

pub trait Middleware {
    fn build(state: &State) -> Result<Self, DependencyInjectionError>
    where
        Self: Sized;
}

pub struct MiddlewareRegistrar {
    pub mw_type_id: std::any::TypeId,
    pub builder: Box<
        dyn Fn(&State) -> Result<Box<dyn Middleware>, DependencyInjectionError>
            + Send
            + Sync,
    >,
}

impl MiddlewareRegistrar {
    pub fn new<M>() -> Self
    where
        M: Middleware + 'static + Send + Sync,
    {
        Self {
            mw_type_id: std::any::TypeId::of::<M>(),
            builder: Box::new(|state: &State| {
                let mw = M::build(state)?;
                Ok(Box::new(mw) as Box<dyn Middleware>)
            }),
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
