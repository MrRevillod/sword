use sword::grpc::*;
use sword::prelude::*;

#[derive(Interceptor)]
pub struct AuthInterceptor;

impl OnRequest for AuthInterceptor {
    async fn on_request(&self, req: Request<()>) -> GrpcInterceptorResult {
        if req.metadata().get("authorization").is_none() {
            return Err(Status::unauthenticated("missing authorization metadata"));
        }

        tracing::info!("Authorization metadata found, proceeding with request");

        Ok(req)
    }
}
