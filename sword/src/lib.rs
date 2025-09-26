//! # Sword - Rust Web Framework
//!
//! Sword is a modern, fast, and ergonomic web framework for Rust, built on top of [Axum](https://github.com/tokio-rs/axum).
//! It provides a clean API, powerful middleware system, and built-in features for rapid web development.
//!
//! ## 🚀 Quick Start
//!
//! ```rust,ignore
//! use sword::prelude::*;
//!
//! #[controller("/api")]
//! struct ApiController;
//!
//! #[routes]
//! impl ApiController {
//!     #[get("/hello")]
//!     async fn hello() -> HttpResponse {
//!         HttpResponse::Ok().message("Hello, World!")
//!     }
//! }
//!
//! #[sword::main]
//! async fn main() {
//!     let app = Application::builder()?
//!         .with_controller::<ApiController>()
//!         .build();
//!     
//!     app.run().await?;
//! }
//! ```
//!
//! ## 🎯 Core Features
//!
//! - ** Macro-based routing** - Clean and intuitive route definitions using `#[get]`, `#[post]`, etc.
//! - ** JSON-first design** - Built-in JSON serialization/deserialization support
//! - ** Request validation** - Automatic validation using `serde` and `validator` crates
//! - ** RFC-compliant HTTP responses** - Standards-compliant HTTP handling
//! - ** Express-like Context** - Rich request context with utility methods
//! - ** Dependency Injection** - Optional DI support using `shaku` crate
//! - ** Middleware system** - Flexible middleware at route and controller levels
//! - ** Async by default** - Built on `tokio` and `axum` for high performance
//!
//! ## 📦 Optional Features
//!
//! Enable additional functionality by adding features to your `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! sword = { version = "0.1.7", features = ["cookies", "multipart", "helmet"] }
//! ```
//!
//! Available features:
//! - `multipart` - File upload support
//! - `cookies` - Cookie handling
//! - `helmet` - Security headers middleware
//! - `shaku-di` - Dependency injection
//!
//! ## 📖 Examples
//!
//! Check out the comprehensive examples in the [repository](https://github.com/sword-framework/sword/tree/main/examples):
//!
//! - **Basic server** - Simple HTTP server setup
//! - **Middleware** - Custom middleware implementation
//! - **Data validation** - Request validation examples
//! - **File uploads** - Multipart form handling
//! - **Dependency injection** - DI patterns
//! - **State management** - Shared application state

mod validation;

/// The prelude module contains the most commonly used items from the Sword framework.
///
/// This module is designed to be imported with a glob import to bring all essential
/// types and traits into scope for typical Sword applications.
///
/// ### Example
///
/// ```rust,ignore
/// use sword::prelude::*;
///
/// // Now you have access to Application, Context, HttpResult, and more
/// ```
pub mod prelude {
    pub use crate::core::{Application, ApplicationConfig};
    pub use crate::core::{ConfigItem, TryFromState, config};

    pub use crate::errors::{ApplicationError, RequestError, StateError};
    pub use crate::web::*;

    #[cfg(feature = "cookies")]
    pub use crate::web::cookies::*;

    #[cfg(feature = "multipart")]
    pub use crate::web::multipart;
}

/// Error types and error handling utilities.
///
/// This module defines the error types used throughout the Sword framework:
///
/// - [`ApplicationError`](errors::ApplicationError) - Errors during application startup and runtime
/// - [`RequestError`](errors::RequestError) - Errors during request processing and validation
/// - [`StateError`](errors::StateError) - Errors when accessing application state
/// - [`ConfigError`](errors::ConfigError) - Configuration-related errors
///
/// All errors implement the standard `Error` trait and provide detailed error messages
/// for debugging and logging purposes.
///
/// ## Error Handling Patterns
///
/// ```rust,ignore
/// use sword::prelude::*;
///
/// #[get("/users/:id")]
/// async fn get_user(ctx: Context) -> HttpResult<HttpResponse> {
///     // This returns a RequestError if parsing fails
///     let user_id = ctx.param::<u32>("id")?;
///     
///     // This returns a StateError if state not found
///     let db = ctx.get_state::<Database>()?;
///     
///     Ok(HttpResponse::Ok().data(user_id))
/// }
/// ```
pub mod errors;

/// Core framework components for application setup and configuration.
///
/// This module contains the fundamental building blocks of a Sword application:
///
/// - [`Application`](core::Application) - The main application struct that manages routing and configuration
/// - [`ApplicationConfig`](core::ApplicationConfig) - Configuration structure for application settings
/// - [`Config`](core::Config) - Configuration management with file and environment variable support
/// - [`State`](core::State) - Thread-safe state container for sharing data across requests
///
/// ## Example
///
/// ```rust,ignore
/// use sword::prelude::*;
///
/// // Create and configure an application
/// let app = Application::builder()?
///     .with_controller::<MyController>()
///     .build();
///
/// // Access configuration
/// let config = app.config.get::<ApplicationConfig>()?;
/// ```
pub mod core {
    mod application;
    mod config;
    mod state;
    mod utils;

    pub use utils::deserialize_size;

    pub use application::{Application, ApplicationConfig};
    pub use config::{Config, ConfigItem, config};
    pub use state::State;
    pub use sword_macros::TryFromState;
}

/// Web-related components for handling HTTP requests and responses.
///
/// This module provides the core web functionality including:
///
/// - [`Context`](web::Context) - Request context containing parameters, headers, body, and utilities
/// - [`Middleware`](web::Middleware) - Trait for implementing custom middleware
/// - HTTP types and utilities from Axum
/// - Routing macros: `#[controller]`, `#[get]`, `#[post]`, etc.
///
/// ## Key Features
///
/// ### Request Handling
/// - Parameter extraction from URL and query strings
/// - JSON request/response handling
/// - Header manipulation
/// - Request validation
///
/// ### Middleware System
/// - Route-level middleware with `#[middleware(MyMiddleware)]`
/// - Controller-level middleware
/// - Built-in middleware for common tasks
///
/// ### Optional Features
/// - **Cookies** - Cookie handling with signed/private support (feature: `cookies`)
/// - **Multipart** - File upload support (feature: `multipart`)
///
/// ## Example
///
/// ```rust,ignore
/// use sword::web::*;
///
/// #[controller("/api")]
/// struct ApiController;
///
/// #[routes]
/// impl ApiController {
///     #[get("/users/:id")]
///     async fn get_user(ctx: Context) -> HttpResult<HttpResponse> {
///         let user_id = ctx.param::<u32>("id")?;
///         // ... fetch user logic
///         Ok(HttpResponse::Ok().message(format!("User {}", user_id)))
///     }
/// }
/// ```
pub mod web {

    mod context;
    mod controller;
    mod middleware;

    pub use axum::http::{Method, StatusCode, header};
    pub use axum_responses::Result as HttpResult;
    pub use axum_responses::http::*;
    pub use sword_macros::{controller, delete, get, patch, post, put, routes};

    pub use crate::next;
    pub use context::{Context, request::RequestValidation};
    pub use middleware::*;

    pub use controller::{Controller, ControllerBuilder, ControllerError};

    #[cfg(feature = "multipart")]
    pub use context::multipart;

    #[cfg(feature = "cookies")]
    pub use context::cookies;
}

pub use sword_macros::main;

#[doc(hidden)]
pub mod __internal {
    pub use axum::body::{Body as AxumBody, HttpBody as AxumHttpBody};
    pub use axum::extract::{FromRequest, FromRequestParts, Request as AxumRequest};
    pub use axum::middleware::Next as AxumNext;
    pub use axum::middleware::from_fn_with_state as mw_with_state;
    pub use axum::response::{IntoResponse, Response as AxumResponse};
    pub use axum::routing::Router as AxumRouter;
    pub use axum::routing::{
        delete as axum_delete_fn, get as axum_get_fn, patch as axum_patch_fn,
        post as axum_post_fn, put as axum_put_fn,
    };

    pub use tokio::runtime as tokio_runtime;
}
