use crate::interceptors::LoggingInterceptor;
use sword::prelude::*;
use sword::web::*;

#[controller(kind = Controller::Web, path = "/")]
#[interceptor(LoggingInterceptor)]
pub struct ExampleRestController;

impl ExampleRestController {
    #[get("/")]
    async fn hello(&self) -> JsonResponse {
        JsonResponse::Ok().message("Hello from sword application!")
    }
}
