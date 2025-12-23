use socketioxide;

pub use rmpv::{Value, ValueRef};
pub use socketioxide::SocketIo;
pub use socketioxide::extract::{AckSender, Data, Event, TryData};

pub use socketioxide::extract::SocketRef;

use crate::core::State;
#[cfg(feature = "socketio")]
use crate::core::{Adapter, HasDeps};

/// Trait for providing WebSocket routes
///
/// This trait is implemented by types annotated with the `#[socketio_adapter]` macro
/// and contains the actual handler implementations for WebSocket events.
#[cfg(feature = "socketio")]
pub trait SocketIoAdapter: HasDeps + Adapter {
    fn path() -> &'static str;
}

#[cfg(feature = "socketio")]
pub type SocketIoSetupFn = Box<dyn Fn(&State)>;

#[doc(hidden)]
#[cfg(feature = "socketio")]
pub mod __internal {
    pub use super::*;
}
