mod schemas;

use schemas::{MyBody, MyQuery};
use sword::prelude::*;

#[controller("/")]
struct AppController {}

#[routes]
impl AppController {
    #[get("/hello")]
    async fn hello(&self, ctx: Context) -> HttpResult {
        match ctx.query_validator::<MyQuery>()? {
            Some(query) => Ok(HttpResponse::Ok()
                .data(query)
                .message("Hello with query parameters")),

            None => Ok(HttpResponse::Ok().message("Hello without query parameters")),
        }
    }

    #[post("/submit")]
    async fn submit_data(&self, ctx: Context) -> HttpResult {
        let body = ctx.body_validator::<MyBody>()?;

        Ok(HttpResponse::Ok()
            .data(body)
            .message("Data submitted successfully"))
    }
}

#[sword::main]
async fn main() {
    let app = Application::builder()
        .with_controller::<AppController>()
        .build();

    app.run().await;
}
