use std::sync::Arc;
use sword::prelude::*;

use crate::{
    chat::{IncommingMessageDto, Message},
    database::Database,
};

#[socketio_adapter("/chat")]
pub struct ChatAdapter {
    db: Arc<Database>,
}

#[handlers]
impl ChatAdapter {
    #[on_connection]
    async fn on_connect(&self) {
        println!("New client connected");
    }

    #[on_message("message")]
    async fn handle_message(&self, ctx: SocketContext) {
        let Ok(data) = ctx.try_validated_data::<IncommingMessageDto>() else {
            eprintln!("Failed to parse message data");
            return;
        };

        let message = Message::from(data);

        self.db.set(&message.id.clone(), message).await;

        let messages = self.db.get_all().await;

        if let Err(e) = ctx.socket.emit("messages", &messages) {
            eprintln!("Failed to emit messages: {:?}", e);
        }
    }
}
