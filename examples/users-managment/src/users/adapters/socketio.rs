use crate::shared::Database;

use std::sync::Arc;
use sword::prelude::*;

#[socketio_adapter("/socket")]
pub struct UserMessagesAdapter {
    db: Arc<Database>,
}

#[handlers]
impl UserMessagesAdapter {
    #[on_connection]
    async fn on_connect(&self) {
        println!("New client connected");
    }

    #[on_message("message-try-data")]
    async fn message_with_try_data(&self, TryData(data): TryData<Value>) {
        match data {
            Ok(value) => println!("Successfully parsed data: {:?}", value),
            Err(e) => println!("Failed to parse data: {:?}", e),
        }
    }

    #[on_message("message-with-ack")]
    async fn message_with_ack(
        &self,
        ack: AckSender,
        Data(_data): Data<Value>,
        Event(_event): Event,
    ) {
        println!("Message with ack received");
        let response = Value::from("Acknowledged!");
        ack.send(&response).ok();
    }

    #[on_message("message-with-event")]
    async fn message_with_event(
        &self,
        Event(event): Event,
        Data(data): Data<Value>,
    ) {
        println!("Message with event '{}' and data: {:?}", event, data);
    }

    #[on_message("another-message")]
    async fn and_another_one_message(&self, _socket: SocketRef, ack: AckSender) {
        println!("Another message received");

        ack.send("response for another-message").ok();
    }

    #[on_message("just-another-message")]
    async fn just_another_message(&self) {
        println!("Message with just-another-message received");
    }

    #[on_disconnection]
    async fn on_disconnect(&self, _socket: SocketRef) {
        println!("Socket disconnected");
    }

    #[on_fallback]
    async fn on_fallback(&self, Event(event): Event, Data(data): Data<Value>) {
        println!(
            "Fallback handler invoked for event: {} with data: {:?}",
            event, data
        );
    }
}
