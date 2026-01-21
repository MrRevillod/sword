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

impl ChatAdapter {
    #[on("connection")]
    async fn on_connect(&self, ctx: SocketContext) {
        println!("New client connected");

        let messages = self.db.get_all().await;

        ctx.socket.emit("messages", &messages).ok();
    }

    #[on("message")]
    async fn handle_message(&self, ctx: SocketContext) {
        let Ok(data) = ctx.try_validated_data::<IncommingMessageDto>() else {
            eprintln!("Failed to validate message data");
            return;
        };

        self.db.set(Message::from(data)).await;

        let messages = self.db.get_all().await;

        ctx.socket.emit("messages", &messages).ok();

        ctx.socket
            .broadcast()
            .emit("messages", &messages)
            .await
            .ok();
    }
}
