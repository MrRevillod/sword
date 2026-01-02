use serde_json::Value;
use sword::prelude::*;

#[derive(Interceptor)]
pub struct LoggingInterceptor;

impl OnConnect for LoggingInterceptor {
    type Error = String;

    async fn on_connect(&self, socket: SocketContext) -> Result<(), Self::Error> {
        println!("[Interceptor] New connection: {}", socket.id());

        match socket.try_data::<Value>() {
            Ok(data) => {
                println!("[Interceptor] Connection data: {:?}", data);
            }
            Err(e) => {
                println!("[Interceptor] Failed to parse auth data: {:?}", e);
            }
        }

        Ok(())
    }
}

#[derive(Interceptor)]
pub struct AuthInterceptor;

impl OnConnect for AuthInterceptor {
    type Error = String;

    async fn on_connect(&self, socket: SocketContext) -> Result<(), Self::Error> {
        println!("[Interceptor] Authenticating socket: {}", socket.id());

        Ok(())
    }
}
