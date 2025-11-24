mod registrar;

pub use axum::middleware::Next;
use axum::response::Response as AxumResponse;

#[doc(hidden)]
pub use registrar::MiddlewareRegistrar;
pub use sword_macros::{middleware, uses};

use crate::core::Build;
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
/// struct AuthMiddleware {}
///
/// impl OnRequest for AuthMiddleware {
///     async fn on_request(&self, req: Request) -> MiddlewareResult {
///         // Middleware logic here
///         req.next().await
///     }
/// }
/// ```
pub trait Middleware: Build {}

/// Trait for middlewares that handle requests
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
    fn on_request(&self, req: Request) -> impl Future<Output = MiddlewareResult>;
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
    ) -> impl Future<Output = MiddlewareResult>;
}
