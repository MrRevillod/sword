pub use socketioxide::{
    ProtocolVersion, SocketIo, TransportType,
    extract::{
        AckSender, Data, Event, Extension, HttpExtension, MaybeExtension,
        MaybeHttpExtension, SocketRef, TryData,
    },
    socket::DisconnectReason,
};

#[cfg(feature = "socketio")]
use crate::core::{Adapter, HasDeps};

/// Trait for providing Socket.IO adapter functionality.
///
/// This trait is implemented by types annotated with the `#[socketio_adapter]` macro
/// and contains the actual handler implementations for Socket.IO events.
#[cfg(feature = "socketio")]
pub trait SocketIoAdapter: HasDeps + Adapter {
    fn namespace() -> &'static str;
}
