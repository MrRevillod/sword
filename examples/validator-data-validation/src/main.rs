mod schemas;

use schemas::{MyBody, MyQuery};
use sword::prelude::*;

#[controller("/")]
struct AppController {}

#[routes]
impl AppController {
    #[get("/hello")]
    async fn hello(&self, req: Request) -> HttpResult {
        match req.query_validator::<MyQuery>()? {
            Some(query) => Ok(HttpResponse::Ok()
                .data(query)
                .message("Hello with query parameters")),

            None => Ok(HttpResponse::Ok().message("Hello without query parameters")),
        }
    }

    #[post("/submit")]
    async fn submit_data(&self, req: Request) -> HttpResult {
        let body = req.body_validator::<MyBody>()?;

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
