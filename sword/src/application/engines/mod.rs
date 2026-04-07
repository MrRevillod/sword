#[cfg(any(feature = "web-controllers", feature = "socketio-controllers"))]
pub mod web;

#[cfg(feature = "grpc-controllers")]
pub mod grpc;

#[cfg(feature = "grpc-controllers")]
use self::grpc::GrpcApplication;
#[cfg(any(feature = "web-controllers", feature = "socketio-controllers"))]
use self::web::WebApplication;

pub enum ApplicationEngine {
    #[cfg(any(feature = "web-controllers", feature = "socketio-controllers"))]
    Web(WebApplication),
    #[cfg(feature = "grpc-controllers")]
    Grpc(GrpcApplication),
}
