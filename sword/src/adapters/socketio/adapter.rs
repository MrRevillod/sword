use super::super::Adapter;

pub use socketioxide::{
    ProtocolVersion, SocketIo, TransportType,
    adapter::LocalAdapter,
    extract::{
        AckSender, Data, Event, Extension, HttpExtension, MaybeExtension,
        MaybeHttpExtension, SocketRef, TryData,
    },
    socket::DisconnectReason,
};

pub use sword_macros::{on, socketio_adapter};

/// Trait for providing Socket.IO adapter functionality.
///
/// This trait is implemented by types annotated with the `#[socketio_adapter]` macro
/// and contains the actual handler implementations for Socket.IO events.
pub trait SocketIoAdapter: Adapter {
    fn namespace() -> &'static str;
}
