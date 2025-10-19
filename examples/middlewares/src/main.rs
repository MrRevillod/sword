use serde_json::json;
use std::sync::Arc;
use sword::prelude::*;

#[middleware]
pub struct ExtensionsTestMiddleware {
    database: Arc<Database>,
}

impl OnRequest for ExtensionsTestMiddleware {
    async fn on_request(&self, mut req: Request, next: Next) -> MiddlewareResult {
        req.extensions
            .insert::<String>("test_extension".to_string());

        next!(req, next)
    }
}

#[middleware]
pub struct MwWithConfig {}

impl OnRequestWithConfig<u8> for MwWithConfig {
    async fn on_request_with_config(
        &self,
        config: u8,
        mut req: Request,
        next: Next,
    ) -> MiddlewareResult {
        req.extensions.insert::<u8>(config);
        next!(req, next)
    }
}

#[controller("/test")]
struct TestController {}

#[routes]
impl TestController {
    #[get("/extensions-test")]
    #[uses(ExtensionsTestMiddleware)]
    async fn extensions_test(&self, req: Request) -> HttpResponse {
        let extension_value = req.extensions.get::<String>();

        HttpResponse::Ok()
            .message("Test controller response with extensions")
            .data(json!({
                "extension_value": extension_value.cloned().unwrap_or_default()
            }))
    }

    #[get("/middleware-config")]
    #[uses(MwWithConfig, config = 1)]
    async fn middleware_config(&self, req: Request) -> HttpResponse {
        let config_value = req.extensions.get::<String>();
        HttpResponse::Ok()
            .message("Test controller response with middleware config")
            .data(json!({
                "config_value": config_value.cloned().unwrap_or_default()
            }))
    }

    #[get("/both")]
    #[uses(ExtensionsTestMiddleware)]
    #[uses(MwWithConfig, config = 1)]
    async fn both_middlewares(&self, req: Request) -> HttpResponse {
        let extension_value = req.extensions.get::<String>();
        let config_value = req.extensions.get::<u8>();

        HttpResponse::Ok()
            .message("Test controller response with both middlewares")
            .data(json!({
                "extension_value": extension_value.cloned().unwrap_or_default(),
                "config_value": config_value.cloned().unwrap_or_default()
            }))
    }
}

#[injectable(kind = "provider")]
pub struct Database {
    pub connection_string: String,
}

impl Database {
    pub fn new() -> Self {
        Self {
            connection_string: "sqlite://:memory:".to_string(),
        }
    }
}

#[sword::main]
async fn main() {
    let db = Database::new();

    let dependency_container =
        DependencyContainer::builder().register_provider(db).build();

    let app = Application::builder()
        .with_dependency_container(dependency_container)
        .with_controller::<TestController>()
        .build();

    app.run().await;
}

impl Default for Database {
    fn default() -> Self {
        Self::new()
    }
}
