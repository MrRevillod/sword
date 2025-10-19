pub mod core;
pub mod prelude;

pub mod web;

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

    pub use crate::core::ConfigRegistrar;

    pub use inventory;

    pub use tokio::runtime as tokio_runtime;

    #[cfg(feature = "hot-reload")]
    pub use dioxus_devtools;
    #[cfg(feature = "hot-reload")]
    pub use subsecond;
}
