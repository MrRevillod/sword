use sword::prelude::*;
use sword::socketio::*;
use sword::web::*;

#[derive(Interceptor)]
pub struct LoggingInterceptor;

impl OnRequest for LoggingInterceptor {
    async fn on_request(&self, req: Request) -> WebInterceptorResult {
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
        println!("[SocketIO] - New connection: - Socket ID: {}", ctx.id());

        Ok(())
    }
}
