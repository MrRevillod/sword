use crate::{adapters::socketio::extract::SocketContext, interceptor::Interceptor};
use socketioxide::adapter::{Adapter as SocketIoAdapter, LocalAdapter};
use std::fmt::Display;

/// An interceptor that is called when a new Socket.IO connection is established.
///
/// ## Example
/// ```rust,ignore
/// use sword::prelude::*;
///
/// #[derive(Interceptor)]
/// pub struct MyInterceptor;
///
/// impl OnConnect for MyInterceptor {
///     type Error = String;
///     async fn on_connect(&self, socket: SocketContext) -> Result<(), Self::Error> {
///         println!("New connection established: {}", socket.id());
///     }
/// }
/// ```
///
/// ## Applying the Interceptor
/// To apply this interceptor use the #[interceptor] attribute under your #[on_connection] method:
/// ```rust,ignore
/// use sword::prelude::*;
///
/// #[socketio_adapter("/my_namespace")]
/// pub struct MyAdapter;
///
/// #[handlers]
/// impl MyAdapter {
///     #[on_connection]
///     #[interceptor(MyInterceptor)]
///     async fn handle_connection(&self, socket: SocketContext) {
///        // handle the initial connection (logging, authentication)
///     }
/// }
/// ```
#[allow(async_fn_in_trait)]
pub trait OnConnect<A = LocalAdapter>: Interceptor
where
    A: SocketIoAdapter,
{
    type Error: Display;

    async fn on_connect(&self, socket: SocketContext<A>) -> Result<(), Self::Error>;
}
