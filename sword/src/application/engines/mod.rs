pub mod web;

#[cfg(any(feature = "web-controllers", feature = "socketio-controllers"))]
use self::web::WebApplication;

pub enum ApplicationEngine {
    #[cfg(any(feature = "web-controllers", feature = "socketio-controllers"))]
    Web(WebApplication),
    // #[allow(dead_code)]
    // #[cfg(feature = "grpc-controllers")]
    // Grpc,
}
