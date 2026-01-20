mod adapter;
mod error;
mod extract;
mod interceptor;

use std::any::TypeId;
use std::pin::Pin;
use std::sync::Arc;
use sword_core::State;

pub use adapter::*;
pub use error::*;
pub use extract::*;
pub use interceptor::*;

#[derive(Clone, Debug)]
pub enum SocketEventKind {
    Connection,
    Disconnection,
    Message(&'static str),
    Fallback,
}

/// Metadata for a SocketIO event handler registered via the `#[on]` attribute.
///
/// This struct is used internally by Sword's auto-registration system to:
/// 1. Discover handlers at compile-time via the `inventory` crate
/// 2. Register them on the appropriate socket at runtime
pub struct HandlerRegistrar {
    /// TypeId of the adapter for filtering during registration
    pub adapter_type_id: TypeId,

    /// Namespace of this adapter (e.g., "/chat")
    pub namespace: &'static str,

    /// Kind of event this handler responds to
    pub event_kind: SocketEventKind,

    /// Name of the handler method
    pub method_name: &'static str,

    /// Registers the handler on the socket (used for Message/Disconnection/Fallback events).
    pub register_fn: fn(Arc<dyn std::any::Any + Send + Sync>, SocketRef),

    /// Executes the handler directly (used for Connection events).
    pub call_fn: fn(
        Arc<dyn std::any::Any + Send + Sync>,
        SocketContext,
    ) -> Pin<Box<dyn Future<Output = ()> + Send>>,
}

/// Setup function that initializes a SocketIO adapter at runtime.
pub struct SocketIoSetupFn {
    /// TypeId of the adapter this setup function handles
    pub adapter_type_id: TypeId,

    /// Setup function that initializes handlers for this adapter
    pub setup: fn(&State),
}

inventory::collect!(HandlerRegistrar);
inventory::collect!(SocketIoSetupFn);
