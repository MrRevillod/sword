use axum_test::TestServer;
use serde_json::json;
use sword::prelude::*;

#[middleware]
struct ExtensionsTestMiddleware;

impl OnRequest for ExtensionsTestMiddleware {
    async fn on_request(&self, mut req: Request) -> MiddlewareResult {
        req.extensions
            .insert::<String>("test_extension".to_string());

        req.next().await
    }
}

#[middleware]
struct MwWithState;

impl OnRequest for MwWithState {
    async fn on_request(&self, mut req: Request) -> MiddlewareResult {
        req.extensions.insert::<u16>(8080);
        req.next().await
    }
}

#[controller("/test")]
#[uses(ExtensionsTestMiddleware)]
struct TestController {}

#[routes]
impl TestController {
    #[get("/extensions-test")]
    async fn extensions_test(&self, req: Request) -> HttpResponse {
        let extension_value = req.extensions.get::<String>();

        HttpResponse::Ok()
            .message("Test controller response with extensions")
            .data(json!({
                "extension_value": extension_value.cloned().unwrap_or_default()
            }))
    }

    #[get("/middleware-state")]
    #[uses(MwWithState)]
    async fn middleware_state(&self, req: Request) -> HttpResult {
        let port = req.extensions.get::<u16>().cloned().unwrap_or(0);
        let message = req.extensions.get::<String>().cloned().unwrap_or_default();

        let json = json!({
            "port": port,
            "message": message
        });

        Ok(HttpResponse::Ok()
            .message("Test controller response with middleware state")
            .data(json))
    }
}

struct TestModule;

impl Module for TestModule {
    type Controller = TestController;
}

#[tokio::test]
async fn extensions_mw_test() {
    let app = Application::builder().with_module::<TestModule>().build();

    let test = TestServer::new(app.router()).unwrap();
    let response = test.get("/test/extensions-test").await;
    assert_eq!(response.status_code(), 200);

    let json = response.json::<ResponseBody>();

    assert!(json.data.is_some());

    let data = json.data.unwrap();

    assert_eq!(data["extension_value"], "test_extension");
}

#[tokio::test]
async fn middleware_state() {
    let app = Application::builder().with_module::<TestModule>().build();

    let test = TestServer::new(app.router()).unwrap();
    let response = test.get("/test/middleware-state").await;

    assert_eq!(response.status_code(), 200);

    let json = response.json::<ResponseBody>();

    assert!(json.data.is_some());

    let data = json.data.unwrap();

    assert_eq!(data["port"], 8080);
    assert_eq!(data["message"], "test_extension");
}
