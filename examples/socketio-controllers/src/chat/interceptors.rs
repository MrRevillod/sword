use sword::prelude::*;

#[derive(Interceptor)]
pub struct LoggingInterceptor;

impl OnRequestWithConfig<&str> for LoggingInterceptor {
    async fn on_request(&self, config: &str, req: Request) -> WebInterceptorResult {
        println!("Using '&str' config with value: {config}");
        req.next().await
    }
}

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

impl OnConnectWithConfig<&str> for LoggingInterceptor {
    type Error = String;

    async fn on_connect(
        &self,
        config: &str,
        ctx: SocketContext,
    ) -> Result<(), Self::Error> {
        println!("[SocketIO] - New connection: - Socket ID: {}", ctx.id());
        println!("Using '&str' config with value: {config}");

        Ok(())
    }
}
