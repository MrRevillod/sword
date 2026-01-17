use crate::interceptors::LoggingInterceptor;
use serde::{Deserialize, Serialize};
use sword::prelude::*;

/// On the Socket.IO adapter the interceptors are only supported on the
/// connection event.

#[derive(Debug, Serialize, Deserialize)]
struct Event {
    pub content: String,
}

#[socketio_adapter("/events")]
pub struct EventsHandler;

#[handlers]
impl EventsHandler {
    #[on_connection]
    #[interceptor(LoggingInterceptor)] // <- Interceptor applied here
    async fn handle_connection(&self, ctx: SocketContext) {
        println!("Client connected: {}", ctx.socket.id);
    }

    #[on_message("eventWithAck")]
    async fn handle_event_ack(
        &self,
        ack: AckSender,
        socket: SocketRef,
        Data(payload): Data<Event>,
    ) {
        println!("Received 'event' from {}: {payload:?}", socket.id);

        ack.send("ok").ok();
    }

    #[on_message("event")]
    async fn handle_event(&self, socket: SocketRef, Data(payload): Data<Event>) {
        println!("Received 'event' from {}: {payload:?}", socket.id);
    }

    #[on_disconnection]
    async fn handle_disconnection(&self, ctx: SocketContext) {
        println!("Client disconnected: {}", ctx.socket.id);
    }
}
