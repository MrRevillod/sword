use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sword::prelude::*;

#[derive(Clone, Serialize, Deserialize)]
#[config(key = "my-custom-section")]
pub struct MyConfig {
    custom_key: String,
}

#[controller("/", version = "v1")]
struct AppController {}

#[routes]
impl AppController {
    #[get("/")]
    async fn get_data(&self) -> HttpResponse {
        let data = vec![
            "This is a basic web server",
            "It serves static data",
            "You can extend it with more routes",
        ];

        HttpResponse::Ok().data(data)
    }

    #[get("/hello")]
    async fn hello(&self) -> HttpResponse {
        HttpResponse::Ok().data("Hello, World!")
    }

    #[post("/submit")]
    async fn submit_data(&self, ctx: Context) -> HttpResult<HttpResponse> {
        let body = ctx.body::<Value>()?;
        let custom_config = ctx.config::<MyConfig>()?;

        Ok(HttpResponse::Ok()
            .data(json!({
                "received_data": body,
                "custom_config": custom_config
            }))
            .message("Data submitted successfully"))
    }

    #[get("/json")]
    async fn get_json(&self) -> HttpResponse {
        HttpResponse::Ok().data(json!({ "foo": "bar" }))
    }
}

#[sword::main]
async fn main() {
    let app = Application::builder()
        .with_controller::<AppController>()
        .build();

    app.run().await;
}
