mod builtin;
mod registrar;

use axum::response::{IntoResponse, Response as AxumResponse};

#[cfg(feature = "helmet")]
pub use builtin::helmet;

pub(crate) use builtin::content_type::ContentTypeCheck;
pub(crate) use builtin::prettifier::ResponsePrettifier;

#[allow(deprecated)]
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
    fn on_request(
        &self,
        req: Request,
        next: Next,
    ) -> impl Future<Output = MiddlewareResult>;
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
    ) -> impl Future<Output = MiddlewareResult>;
}

pub struct SwordNext {
    inner: axum::middleware::Next,
}

impl SwordNext {
    pub fn new(inner: axum::middleware::Next) -> Self {
        Self { inner }
    }

    pub async fn run(self, req: Request) -> AxumResponse {
        let Ok(axum_req) = req.try_into() else {
            return HttpResponse::InternalServerError().into_response();
        };

        self.inner.run(axum_req).await
    }
}

/// A macro to simplify the next middleware call in the middleware chain.
///
/// Deprecated: Use the `req.run(next).await` method instead.
/// This macro will be removed in future versions.
///
/// # Example usage:
/// ```rust,ignore
/// use sword::prelude::*;
///
/// struct MyMiddleware;
///
/// impl Middleware for MyMiddleware {
///     async fn handle(req: Request, next: Next) -> MiddlewareResult {
///         // before: next!(ctx, next)
///         // now: req.run(next).await
///     }
/// }
#[deprecated(note = "Use the `req.run(next).await` method instead.")]
#[macro_export]
macro_rules! next {
    ($req:expr, $next:expr) => {
        Ok($next.run($req.try_into()?).await)
    };
}
