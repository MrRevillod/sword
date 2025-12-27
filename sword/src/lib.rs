mod application;
mod module;

pub mod adapters;
pub mod interceptors;
pub mod prelude;

pub use application::*;
pub use sword_macros::main;

pub mod layers {
    pub use sword_layers::helmet;
}

#[doc(hidden)]
pub mod internal {
    pub mod axum {
        pub use axum::body::{Body as AxumBody, HttpBody as AxumHttpBody};
        pub use axum::extract::{
            FromRequest, FromRequestParts, Request as AxumRequest,
        };
        pub use axum::middleware::{
            Next as AxumNext, from_fn_with_state as mw_with_state,
        };
        pub use axum::response::{IntoResponse, Response as AxumResponse};
        pub use axum::routing::{
            Router as AxumRouter, delete as delete_fn, get as get_fn,
            patch as patch_fn, post as post_fn, put as put_fn,
        };
    }

    pub mod core {
        pub use crate::interceptors::MiddlewareRegistrar;
        pub use sword_core::*;
    }

    pub use inventory;
    pub use tokio::runtime as tokio_runtime;

    pub use tracing;

    #[cfg(feature = "hot-reload")]
    pub use dioxus_devtools;

    #[cfg(feature = "hot-reload")]
    pub use subsecond;
}
