use sword::{web_socket, web_socket_gateway};

use crate::shared::Database;
use std::sync::Arc;

#[web_socket_gateway]
pub struct UserMessagesAdapter {
    db: Arc<Database>,
}

#[web_socket("/socket")]
impl UserMessagesAdapter {
    #[on_connection]
    async fn on_connect(&self, _socket: SocketRef) {
        println!("New client connected");
    }

    #[subscribe_message("message")]
    async fn on_message(&self, _socket: SocketRef, Data(_data): Data<Value>) {
        println!("New message received");

        let now = sqlx::query("SELECT NOW() as now")
            .fetch_one(self.db.get_pool())
            .await
            .expect("Oh no");

        println!("Database time: {:?}", now);
    }

    #[subscribe_message("message2")]
    async fn other_message(&self, _socket: SocketRef, Data(_data): Data<Value>) {
        println!("Other message received");
    }

    #[subscribe_message("message-with-ack")]
    async fn message_with_ack(
        &self,
        Event(_event): Event,
        Data(_data): Data<Value>,
        ack: AckSender,
    ) {
        println!("Message with ack received");
        let response = Value::from("Acknowledged!");
        ack.send(&response).ok();
    }

    #[subscribe_message("message-with-event")]
    async fn message_with_event(
        &self,
        Event(event): Event,
        Data(data): Data<Value>,
    ) {
        println!("Message with event '{}' and data: {:?}", event, data);
    }

    #[subscribe_message("another-message")]
    async fn and_another_one_message(&self, _socket: SocketRef, ack: AckSender) {
        println!("Another message received");

        ack.send("response for another-message").ok();
    }

    #[subscribe_message("just-another-message")]
    async fn just_another_message(&self) {
        println!("Message with just-another-message received");
    }

    #[on_disconnect]
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
