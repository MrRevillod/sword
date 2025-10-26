//! # Sword - Rust Web Framework
//!
//! Sword is a modern, fast, and ergonomic web framework for Rust, built on top of [Axum](https://github.com/tokio-rs/axum).
//! It provides a clean API, powerful middleware system, and built-in features for rapid web development.
//!
//! ## Quick Start
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
//!     async fn hello(&self) -> HttpResponse {
//!         HttpResponse::Ok().message("Hello, World!")
//!     }
//! }
//!
//! #[sword::main]
//! async fn main() {
//!     let app = Application::builder()
//!         .with_controller::<ApiController>()
//!         .build();
//!     
//!     app.run().await;
//! }
//! ```
//!
//! ## Core Features
//!
//! - ** Macro-based routing** - Clean and intuitive route definitions using `#[get]`, `#[post]`, etc.
//! - ** JSON-first design** - Built-in JSON serialization/deserialization support
//! - ** Request validation** - Automatic validation using `serde` and `validator` crates
//! - ** RFC-compliant HTTP responses** - Standards-compliant HTTP handling
//! - ** Express-like Request** - Rich request struct with utility methods
//! - ** Dependency Injection** - Built-in DI system for managing application state
//! - ** Middleware system** - Flexible middleware at route and controller levels
//! - ** Async by default** - Built on `tokio` and `axum` for high performance
//!
//! ## Optional Features
//!
//! Enable additional functionality by adding features to your `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! sword = { version = "0.1.9", features = ["cookies", "multipart", "helmet"] }
//! ```
//!
//! Available features:
//! - `multipart` - File upload support
//! - `cookies` - Cookie handling
//! - `helmet` - Security headers middleware

/// The prelude module contains the most commonly used items from the Sword framework.
///
/// This module is designed to be imported with a glob import to bring all essential
/// types and traits into scope for typical Sword applications.
///
/// ### Example
///
/// ```rust,ignore
/// use sword::prelude::*;
/// ```
pub use crate::core::*;
pub use crate::web::*;

#[cfg(feature = "cookies")]
pub use crate::web::cookies::*;

#[cfg(feature = "multipart")]
pub use crate::web::multipart;

#[cfg(feature = "websocket")]
pub use crate::web::websocket::*;

#[cfg(feature = "validator")]
pub use crate::web::request_validator::*;
