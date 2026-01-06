use std::sync::Arc;

use sword::prelude::*;

#[derive(Interceptor)]
pub struct LoggingInterceptor {
    pub a: Arc<String>,
}

impl OnRequest for LoggingInterceptor {
    async fn on_request(&self, req: Request) -> HttpInterceptorResult {
        println!(
            "[REST] - Incoming request: ID: {} - [{}] {}",
            req.id(),
            req.method(),
            req.uri()
        );

        req.next().await
    }
}

impl OnConnect for LoggingInterceptor {
    type Error = String;

    async fn on_connect(&self, ctx: SocketContext) -> Result<(), Self::Error> {
        println!(
            "[SocketIO] - New connection: - Socket ID: {}",
            ctx.socket.id
        );

        Ok(())
    }
}
