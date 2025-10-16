mod cors;
use crate::cors::{CorsConfig, CorsMiddleware};

use sword::prelude::*;
use tokio::time::{sleep, Duration};
use tower_http::timeout::TimeoutLayer;

#[controller("/")]
struct AppController {}

#[routes]
impl AppController {
    #[get("/")]
    #[middleware(TimeoutLayer::new(Duration::from_secs(2)))]
    async fn get_data(&self) -> HttpResponse {
        sleep(Duration::from_secs(3)).await;
        HttpResponse::Ok()
    }

    #[get("/fast")]
    async fn get_fast_data(&self) -> HttpResponse {
        HttpResponse::Ok()
    }
}

#[sword::main]
async fn main() {
    let mut app = Application::builder();

    let cors_config = app
        .config::<CorsConfig>()
        .expect("Failed to load CORS config");

    let cors_middleware = CorsMiddleware::new(&cors_config);

    app = app
        .with_controller::<AppController>()
        .with_layer(cors_middleware.layer);

    app.build().run().await;
}
