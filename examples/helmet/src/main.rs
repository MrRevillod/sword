use sword::prelude::*;
use sword::web::helmet::*;

#[controller("/")]
struct MyController;

#[routes]
impl MyController {
    #[get("/")]
    async fn index(&self) -> HttpResult {
        Ok(HttpResponse::Ok().message("Hello, Helmet!"))
    }
}

#[sword::main]
async fn main() {
    let helmet = Helmet::builder()
        .with_header(XContentTypeOptions::nosniff())
        .with_header(XXSSProtection::on())
        .build();

    let app = Application::builder()
        .with_controller::<MyController>()
        .with_layer(helmet)
        .build();

    app.run().await;
}
