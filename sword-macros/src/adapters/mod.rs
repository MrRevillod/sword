#[cfg(feature = "adapter-http-controllers")]
pub mod http;

#[cfg(feature = "adapter-http-controllers")]
pub use http::*;

#[cfg(feature = "adapter-socketio")]
mod socketio;

#[cfg(feature = "adapter-socketio")]
pub use socketio::*;
