use super::super::Adapter as SwordAdapter;
use sword_core::HasDeps;

pub use socketioxide::{
    ProtocolVersion, SocketIo, TransportType,
    adapter::Adapter as SocketIoxideAdapterType,
    adapter::LocalAdapter,
    extract::{
        AckSender, Data, Event, Extension, HttpExtension, MaybeExtension,
        MaybeHttpExtension, SocketRef, TryData,
    },
    handler::ConnectHandler,
    socket::DisconnectReason,
};

pub use sword_macros::{
    handlers, on_connection, on_disconnection, on_fallback, on_message,
    socketio_adapter,
};

// type OnConnectFn<A> = Box<
//     dyn Fn(
//             SocketContext<A>,
//         ) -> Pin<
//             Box<dyn Future<Output = Result<(), Box<dyn std::error::Error>>> + Send>,
//         > + Send
//         + Sync,
// >;

/// Trait for providing Socket.IO adapter functionality.
///
/// This trait is implemented by types annotated with the `#[socketio_adapter]` macro
/// and contains the actual handler implementations for Socket.IO events.
pub trait SocketIoAdapter: HasDeps + SwordAdapter {
    fn namespace() -> &'static str;
    // fn interceptors<A: SocketIoxideAdapterType>() -> Vec<OnConnectFn<A>> {
    //     Vec::new()
    // }
}
