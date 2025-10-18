mod builtin;

use std::{any::Any, sync::Arc};

use axum::response::Response as AxumResponse;
use axum_responses::http::HttpResponse;

#[cfg(feature = "helmet")]
pub use builtin::helmet;

pub(crate) use builtin::content_type::ContentTypeCheck;
pub(crate) use builtin::prettifier::ResponsePrettifier;

pub use axum::middleware::Next;
pub use sword_macros::{middleware, on_request, uses};

use crate::{core::State, errors::DependencyInjectionError};

pub type MiddlewareResult = Result<AxumResponse, HttpResponse>;

pub trait Middleware: Any + Send + Sync + 'static {
    fn build(state: &State) -> Result<Self, DependencyInjectionError>
    where
        Self: Sized;
}

pub struct MiddlewareRegistrar {
    pub build: fn(&State) -> Result<Arc<dyn Middleware>, DependencyInjectionError>,
}

impl MiddlewareRegistrar {
    pub const fn new<M>() -> Self
    where
        M: Middleware + Clone,
    {
        Self {
            build: |state: &State| {
                Ok(Arc::new(M::build(state)?) as Arc<dyn Middleware>)
            },
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
