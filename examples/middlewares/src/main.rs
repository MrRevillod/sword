use serde_json::json;
use sword::prelude::*;

mod middlewares;
// use middlewares::*;

#[controller("/test")]
// #[middleware(LoggerMiddleware)]
struct TestController {}

#[routes]
impl TestController {
    #[get("/extensions-test")]
    // #[middleware(ExtensionsTestMiddleware)]
    async fn extensions_test(&self, ctx: Context) -> HttpResponse {
        let extension_value = ctx.extensions.get::<String>();

        HttpResponse::Ok()
            .message("Test controller response with extensions")
            .data(json!({
                "extension_value": extension_value.cloned().unwrap_or_default()
            }))
    }

    #[get("/role-test")]
    // #[middleware(ExtensionsTestMiddleware)]
    // #[middleware(RoleMiddleware, config = vec!["admin", "user"])]
    async fn role_test(&self) -> HttpResponse {
        HttpResponse::Ok().message("Role middleware test passed")
    }

    #[get("/error-test")]
    // #[middleware(ErrorMiddleware)]
    async fn error_test(&self) -> HttpResponse {
        HttpResponse::Ok()
    }
}

#[sword::main]
async fn main() {
    let app = Application::builder()
        .with_controller::<TestController>()
        .build();

    app.run().await;
}
