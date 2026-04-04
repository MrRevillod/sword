#![allow(async_fn_in_trait)]

mod application;
mod controllers;
mod interceptor;
mod module;
pub mod prelude;

pub use application::*;
pub use sword_macros::main;

#[doc(hidden)]
pub mod internal {
    pub mod axum {
        pub use axum::body::{Body as AxumBody, HttpBody as AxumHttpBody};
        pub use axum::extract::{FromRequest, FromRequestParts, Request as AxumRequest};
        pub use axum::middleware::{Next as AxumNext, from_fn_with_state as mw_with_state};
        pub use axum::response::{IntoResponse, Response as AxumResponse};
        pub use axum::routing;
        pub use axum::routing::{
            MethodRouter, Router as AxumRouter, delete, delete as delete_fn, get, get as get_fn,
            patch, patch as patch_fn, post, post as post_fn, put, put as put_fn,
        };
    }

    #[cfg(feature = "socketio-controllers")]
    pub mod socketio {
        pub use crate::controllers::socketio::{
            HandlerRegistrar, SocketEventKind, SocketIoHandlerRegistrar,
        };
        pub use socketioxide::SocketError;
        pub use socketioxide::handler::ConnectHandler;
        pub use socketioxide::handler::connect::FromConnectParts;
    }

    pub mod core {
        #[cfg(feature = "socketio-controllers")]
        pub use crate::controllers::socketio::SocketIoController;
        #[cfg(feature = "web-controllers")]
        pub use crate::controllers::web::WebController;
        pub use crate::controllers::{Controller, ControllerSpec};
        pub use crate::interceptor::{Interceptor, InterceptorRegistrar};
        #[doc(hidden)]
        pub use sword_core::sword_error;
        pub use sword_core::*;
    }

    #[cfg(feature = "web-controllers")]
    pub mod controllers {
        pub use crate::controllers::web::RouteRegistrar;
        pub use crate::controllers::web::WebControllerRegistrar;
    }

    pub use inventory;
    pub use tokio::runtime as tokio_runtime;

    pub use tracing;

    #[cfg(feature = "hot-reload")]
    pub use dioxus_devtools;

    #[cfg(feature = "hot-reload")]
    pub use subsecond;
}
