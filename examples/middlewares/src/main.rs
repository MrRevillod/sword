use std::sync::Arc;

use serde_json::json;
use sword::prelude::*;

#[middleware]
pub struct ExtensionsTestMiddleware {
    database: Arc<Database>,
}

impl ExtensionsTestMiddleware {
    #[on_request]
    async fn add_extension(&self, mut ctx: Context, next: Next) -> MiddlewareResult {
        ctx.extensions
            .insert::<String>("test_extension".to_string());

        next!(ctx, next)
    }
}

#[controller("/test")]
struct TestController {}

#[routes]
impl TestController {
    #[get("/extensions-test")]
    #[uses(ExtensionsTestMiddleware)]
    async fn extensions_test(&self, ctx: Context) -> HttpResponse {
        let extension_value = ctx.extensions.get::<String>();

        HttpResponse::Ok()
            .message("Test controller response with extensions")
            .data(json!({
                "extension_value": extension_value.cloned().unwrap_or_default()
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
