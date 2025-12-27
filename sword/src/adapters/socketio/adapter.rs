use super::super::Adapter;
use sword_core::HasDeps;

pub use socketioxide::{
    ProtocolVersion, SocketIo, TransportType,
    extract::{
        AckSender, Data, Event, Extension, HttpExtension, MaybeExtension,
        MaybeHttpExtension, SocketRef, TryData,
    },
    socket::DisconnectReason,
};

pub use sword_macros::{
    handlers, on_connection, on_disconnection, on_fallback, on_message,
    socketio_adapter,
};

/// Trait for providing Socket.IO adapter functionality.
///
/// This trait is implemented by types annotated with the `#[socketio_adapter]` macro
/// and contains the actual handler implementations for Socket.IO events.
#[cfg(feature = "adapter-socketio")]
pub trait SocketIoAdapter: HasDeps + Adapter {
    fn namespace() -> &'static str;
}
