mod builtin;
mod registrar;

use axum::response::Response as AxumResponse;

#[cfg(feature = "helmet")]
pub use builtin::helmet;

pub(crate) use builtin::content_type::ContentTypeCheck;
pub(crate) use builtin::prettifier::ResponsePrettifier;

pub use crate::next;
pub use axum::middleware::Next;

#[doc(hidden)]
pub use registrar::MiddlewareRegistrar;
pub use sword_macros::{middleware, uses};

use crate::core::{Clonable, DependencyInjectionError};
use crate::web::{HttpResponse, Request};
use std::future::Future;

pub type MiddlewareResult = Result<AxumResponse, HttpResponse>;

/// Trait for middleware components that can intercept and modify requests/responses.
///
/// Middlewares in Sword must be cloneable to be shared across multiple requests
/// efficiently. They are automatically constructed from the application State
/// during initialization.
///
/// Use the `#[middleware]` macro to automatically implement this trait.
///
/// # Example
///
/// ```rust,ignore
/// use sword::prelude::*;
///
/// #[middleware]
/// struct AuthMiddleware {
///     secret: String,
/// }
///
/// impl OnRequest for AuthMiddleware {
///     async fn on_request(&self, req: Request, next: Next) -> MiddlewareResult {
///         // Middleware logic here
///         next!(req, next)
///     }
/// }
/// ```
pub trait Middleware: Clonable<Error = DependencyInjectionError> {}

/// Trait for middlewares that handle requests without configuration.
///
/// This is the standard middleware trait for simple request interception.
/// Implement this trait to create middlewares that don't require additional
/// configuration at the route level.
pub trait OnRequest: Middleware {
    /// Handles an incoming request.
    ///
    /// This method receives the request and the next middleware in the chain.
    /// It should either call `next` to continue the chain or return early with
    /// a response to short-circuit the request.
    fn on_request(
        &self,
        req: Request,
        next: Next,
    ) -> impl Future<Output = MiddlewareResult> + Send;
}

/// Trait for middlewares that handle requests with route-specific configuration.
///
/// This trait allows middlewares to receive configuration parameters when applied
/// to specific routes. The configuration type `C` is provided at compile time
/// through the `#[uses]` attribute.
pub trait OnRequestWithConfig<C>: Middleware {
    /// Handles an incoming request with configuration.
    ///
    /// This method receives configuration, the request, and the next middleware.
    /// The configuration is passed from the route definition and can be used to
    /// customize middleware behavior per route.
    fn on_request_with_config(
        &self,
        config: C,
        req: Request,
        next: Next,
    ) -> impl Future<Output = MiddlewareResult> + Send;
}

/// A macro to simplify the next middleware call in the middleware chain.
///
/// It takes the current Request and the next middleware in the chain,
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
///     async fn handle(req: Request, next: Next) -> MiddlewareResult {
///         next!(ctx, next)
///     }
/// }
#[macro_export]
macro_rules! next {
    ($req:expr, $next:expr) => {
        Ok($next.run($req.try_into()?).await)
    };
}
