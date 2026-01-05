use crate::interceptors::LoggingInterceptor;
use sword::prelude::*;

/// On the REST adapter the interceptors can be applied at the
/// controller level to affect all routes within the controller, also
/// can be applied at the route level to affect only that specific route.
///
/// For REST Adapter, it's valid to use `#[controller]` or `#[rest_adapter]`

#[controller("/")]
#[interceptor(LoggingInterceptor)]
pub struct ExampleRestController;

#[routes]
impl ExampleRestController {
    #[get("/")]
    async fn hello(&self) -> JsonResponse {
        JsonResponse::Ok().message("Hello from sword application!")
    }
}
