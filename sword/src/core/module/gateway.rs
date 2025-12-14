use crate::core::State;
use axum::Router;
use parking_lot::RwLock;

pub(crate) type WithStateRouterBuilder = Box<dyn Fn(State) -> Router>;

pub enum GatewayKind {
    Rest(WithStateRouterBuilder),
    WebSocket(WithStateRouterBuilder),
    Grpc,
}

pub trait Gateway {
    fn kind() -> GatewayKind;
}

pub struct GatewayRegistry {
    pub(crate) gateways: RwLock<Vec<GatewayKind>>,
}

impl GatewayRegistry {
    pub fn new() -> Self {
        Self {
            gateways: RwLock::new(Vec::new()),
        }
    }

    pub fn register<G: Gateway>(&self) {
        self.gateways.write().push(G::kind());
    }
}

impl Default for GatewayRegistry {
    fn default() -> Self {
        Self::new()
    }
}
