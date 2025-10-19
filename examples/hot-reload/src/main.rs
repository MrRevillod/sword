use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sword::prelude::*;

#[derive(Clone, Deserialize, Debug, Serialize)]
#[config(key = "my-custom-section")]
pub struct MyConfig {
    custom_key: String,
}

#[controller("/", version = "v1")]
struct AppController {
    custom_config: MyConfig,
}

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
    async fn submit_data(&self, req: Request) -> HttpResult {
        let body = req.body::<Value>()?;

        Ok(HttpResponse::Ok()
            .data(json!({
                "received_data": body,
                "custom_config": &self.custom_config
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
