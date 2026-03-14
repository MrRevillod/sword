use super::super::ControllerSpec;

pub use socketioxide::{
    ProtocolVersion, SocketIo, TransportType,
    adapter::LocalAdapter,
    extract::{
        AckSender, Data, Event, Extension, HttpExtension, MaybeExtension,
        MaybeHttpExtension, SocketRef, TryData,
    },
    socket::DisconnectReason,
};

pub use sword_macros::on;

/// Trait for providing Socket.IO controller functionality.
///
/// This trait is implemented by types annotated with
/// `#[controller(kind = Controller::SocketIo, namespace = "...")]`
/// and contains the actual handler implementations for Socket.IO events.
pub trait SocketIoController: ControllerSpec {
    fn namespace() -> &'static str;
}
