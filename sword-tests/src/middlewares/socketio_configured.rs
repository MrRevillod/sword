use axum_test::TestServer;
use sword::prelude::*;

#[derive(Interceptor)]
struct ConfiguredSocketInterceptor;

impl OnConnectWithConfig<&str> for ConfiguredSocketInterceptor {
    type Error = String;

    async fn on_connect(
        &self,
        config: &str,
        _ctx: SocketContext,
    ) -> Result<(), Self::Error> {
        let _ = config;
        Ok(())
    }
}

#[controller(kind = Controller::SocketIo, namespace = "/configured")]
#[interceptor(ConfiguredSocketInterceptor, config = "socket-config")]
struct ConfiguredSocketController;

impl ConfiguredSocketController {
    #[on("connection")]
    async fn on_connect(&self, _ctx: SocketContext) {}
}

struct TestModule;

impl Module for TestModule {
    fn register_controllers(controllers: &ControllerRegistry) {
        controllers.register::<ConfiguredSocketController>();
    }
}

#[tokio::test]
async fn socketio_controller_with_configured_interceptor_builds() {
    let app = Application::builder().with_module::<TestModule>().build();
    let test = TestServer::new(app.router()).unwrap();

    let response = test.get("/socket.io/?EIO=4&transport=polling").await;

    assert_eq!(response.status_code(), 200);
}
