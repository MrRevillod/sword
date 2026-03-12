use crate::interceptors::LoggingInterceptor;
use sword::prelude::*;

#[controller("/")]
#[interceptor(LoggingInterceptor)]
pub struct ExampleRestController;

impl ExampleRestController {
    #[get("/")]
    #[interceptor(LoggingInterceptor)]
    async fn hello(&self) -> JsonResponse {
        JsonResponse::Ok().message("Hello from sword application!")
    }
}
