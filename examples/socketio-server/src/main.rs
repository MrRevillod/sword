use dotenv::dotenv;
use std::sync::Arc;
use sword::prelude::*;
use sword_macros::{
    on_connection, on_disconnect, on_fallback, subscribe_message, web_socket,
    web_socket_gateway,
};

use crate::database::{Database, DatabaseConfig};
mod database;

#[controller("/ohno")]
struct AppController {}

#[routes]
impl AppController {
    #[get("/test")]
    async fn get_data(&self, _req: Request) -> HttpResponse {
        let data = vec![
            "This is a basic web server",
            "It serves static data",
            "You can extend it with more routes",
        ];

        HttpResponse::Ok().data(data)
    }
}

#[web_socket_gateway]
struct SocketController {
    db: Arc<Database>,
}

#[web_socket_gateway]
struct OtherSocketController {}

#[web_socket("/other_socket")]
impl OtherSocketController {
    #[on_connection]
    async fn on_connect(&self, _socket: SocketRef) {
        println!("New client connected to OtherSocketController");
    }
}

#[web_socket("/socket")]
impl SocketController {
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

#[sword::main]
async fn main() {
    dotenv().ok();

    let app = Application::builder();
    let db_config = app.config::<DatabaseConfig>().unwrap();
    let db = Database::new(db_config).await;

    let container = DependencyContainer::builder().register_provider(db).build();

    let app = Application::builder()
        .with_dependency_container(container)
        .with_socket::<OtherSocketController>()
        .with_socket::<SocketController>()
        .with_controller::<AppController>()
        .build();

    app.run().await;
}
