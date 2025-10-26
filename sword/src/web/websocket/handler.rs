//! WebSocket handler traits and utilities

use crate::core::State;
use axum::routing::Router;
#[cfg(feature = "websocket")]
use socketioxide::SocketIo;
use std::sync::Arc;

/// Trait for WebSocket gateway controllers
///
/// This trait is automatically implemented by types annotated with `#[web_socket_gateway]`
/// and provides the necessary methods for registering WebSocket handlers with the application.
#[cfg(feature = "websocket")]
pub trait WebSocketGateway: Send + Sync + 'static {
    /// Creates a router for the WebSocket handlers
    fn router(state: State) -> Router;
}

/// Trait for providing WebSocket routes
///
/// This trait is implemented by types annotated with the `#[web_socket]` macro
/// and contains the actual handler implementations for WebSocket events.
#[cfg(feature = "websocket")]
pub trait WebSocketProvider: Send + Sync + 'static {
    /// Returns the path where the WebSocket is mounted
    fn path() -> &'static str;

    /// Returns a setup function that configures handlers on the SocketIo instance
    /// This should register the namespace and set up all handlers
    fn get_setup_fn(state: State) -> SocketSetupFn;

    /// Creates a router for the WebSocket
    fn router(state: State) -> Router;
}

/// Type for Socket.IO setup functions
/// Takes both SocketIo reference and the application state for dependency injection
#[cfg(feature = "websocket")]
pub type SocketSetupFn = Arc<dyn Fn(&SocketIo, State) + Send + Sync>;
