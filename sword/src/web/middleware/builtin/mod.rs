pub(crate) mod content_type;

/// Module containing various security headers that can be added to HTTP responses.
/// These headers help protect against common web vulnerabilities.
///
/// This module re-exports headers from the `rust-helmet` and `axum-helmet` crate
/// for easy access. You can use these headers to configure the `Helmet` layer.
///
/// Documentation for each header can be found in the `axum-helmet` crate.
/// [axum-helmet docs](https://docs.rs/axum-helmet/latest/axum_helmet/)
/// [axum-helmet crate](https://crates.io/crates/axum-helmet)
#[cfg(feature = "helmet")]
pub mod helmet;

pub mod cors;
