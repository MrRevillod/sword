//! SocketIO support for Sword.
//!
//! This module provides types and traits for handling SocketIO connections
//! via `socketioxide`.
//!
//! # Example
//!
//! ```rust,ignore
//! use sword::prelude::*;
//!
//! #[web_socket_gateway]
//! struct ChatSocket;
//!
//! #[web_socket("/chat")]
//! impl ChatSocket {
//!     #[on_connection]
//!     async fn on_connect(&self, socket: SocketRef) {
//!         println!("Client connected: {}", socket.id);
//!     }
//!
//!     #[subscribe_message("message")]
//!     async fn on_message(&self, socket: SocketRef, Data(msg): Data<String>) {
//!         socket.inner().emit("message", msg).ok();
//!     }
//!
//!     #[on_disconnect]
//!     async fn on_disconnect(&self, socket: WebSocket) {
//!         println!("Client disconnected: {}", socket.id());
//!     }
//! }
//! ```

pub mod handler;
pub mod types;

pub use handler::*;
pub use types::*;
