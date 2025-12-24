use crate::shared::Database;

use serde::Deserialize;
use std::sync::Arc;
use sword::prelude::*;

#[socketio_adapter("/socket")]
pub struct UserMessagesAdapter {
    db: Arc<Database>,
}

#[derive(Deserialize, Debug)]
struct MessagePayload {
    content: String,
}

#[handlers]
impl UserMessagesAdapter {
    #[on_connection]
    async fn on_connect(&self) {
        println!("New client connected");
    }

    #[on_message("message-try-data")]
    async fn message_with_try_data(&self, TryData(data): TryData<MessagePayload>) {
        match data {
            Ok(value) => {
                println!("Successfully parsed data - content: {}", value.content)
            }
            Err(e) => println!("Failed to parse data: {:?}", e),
        }
    }

    #[on_message("message-with-ack")]
    async fn message_with_ack(
        &self,
        ack: AckSender,
        Data(data): Data<Value>,
        Event(event): Event,
    ) {
        println!(
            "Event: {event} - Message with ack received - title: {}, count: {}",
            data["title"], data["count"]
        );

        ack.send("acknowledged").ok();
    }

    #[on_message("another-message")]
    async fn and_another_one_message(&self, ack: AckSender) {
        println!("Another message received");

        ack.send("response for another-message").ok();
    }

    #[on_disconnection]
    async fn on_disconnect(&self) {
        println!("Socket disconnected");
    }

    #[on_fallback]
    async fn on_fallback(&self, Event(event): Event, Data(data): Data<Value>) {
        println!("Fallback handler invoked for event: {event} with data: {data:?}",);
    }
}
