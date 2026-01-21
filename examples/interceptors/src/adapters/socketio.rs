use serde::{Deserialize, Serialize};
use sword::prelude::*;

use crate::interceptors::LoggingInterceptor;

/// Socket.IO adapter example with auto-registration and interceptors.
///
/// The `#[interceptor]` attribute can be used on the adapter to apply interceptors
/// to all connection events. The interceptor is applied using `.with()` from socketioxide.
/// The connection handler method must be named `on_connect` for auto-registration.
/// Message handlers can have any name, but each handles a specific event.

#[derive(Debug, Serialize, Deserialize)]
struct Event {
    pub content: String,
}

#[socketio_adapter("/events")]
#[interceptor(LoggingInterceptor)]
pub struct EventsHandler;

impl EventsHandler {
    #[on("connection")]
    async fn on_connect_method(&self, ctx: SocketContext) {
        println!("Client connected: {}", ctx.socket.id);
    }

    #[on("event")]
    async fn handle_message_event(&self, ctx: SocketContext) {
        let payload: Event = ctx.try_data().expect("Failed to parse event data");

        println!("Received 'event' from {}: {payload:?}", ctx.socket.id);
    }

    #[on("eventWithAck")]
    async fn handle_message(&self, ctx: SocketContext) {
        let payload: Event = ctx.try_data().expect("Failed to parse event data");

        println!(
            "Received 'eventWithAck' from {}: {payload:?}",
            ctx.socket.id
        );

        if ctx.has_ack() {
            ctx.ack("ok").ok();
        }
    }
}
