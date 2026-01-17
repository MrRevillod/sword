mod adapters;
mod application;
mod interceptor;
mod module;

pub use application::*;
pub mod prelude;

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
        pub use axum::routing;
        pub use axum::routing::{
            MethodRouter, Router as AxumRouter, delete, delete as delete_fn, get,
            get as get_fn, patch, patch as patch_fn, post, post as post_fn, put,
            put as put_fn,
        }; // Export routing module for macros
    }

    #[cfg(feature = "adapter-socketio")]
    pub mod socketio {
        pub use socketioxide::SocketError;
        pub use socketioxide::handler::ConnectHandler;
        pub use socketioxide::handler::connect::FromConnectParts;
    }

    pub mod core {
        pub use crate::adapters::rest::RestAdapter;
        #[cfg(feature = "adapter-socketio")]
        pub use crate::adapters::socketio::SocketIoAdapter;
        pub use crate::adapters::{Adapter, AdapterKind};
        pub use crate::interceptor::{Interceptor, InterceptorRegistrar};
        pub use sword_core::*;
    }

    pub mod rest {
        pub use crate::adapters::rest::RouteRegistrar;
    }

    pub use inventory;
    pub use tokio::runtime as tokio_runtime;

    pub use tracing;

    #[cfg(feature = "hot-reload")]
    pub use dioxus_devtools;

    #[cfg(feature = "hot-reload")]
    pub use subsecond;
}
