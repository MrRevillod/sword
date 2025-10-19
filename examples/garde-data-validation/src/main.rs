mod extractors;
mod schemas;

use schemas::MyBody;
use sword::prelude::*;

use crate::extractors::GardeRequestValidation;

#[controller("/")]
struct AppController {}

#[routes]
impl AppController {
    #[post("/submit")]
    async fn submit_data(&self, req: Request) -> HttpResult {
        let body = req.body_garde::<MyBody>()?;

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
