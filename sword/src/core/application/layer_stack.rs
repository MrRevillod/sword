use axum::{
    Router, extract::Request as AxumRequest, response::IntoResponse, routing::Route,
};
use std::convert::Infallible;
use tower::{Layer, Service};

pub(crate) type LayerFn = Box<dyn Fn(Router) -> Router + Send + Sync>;

/// A stack for managing and applying middleware layers to a router.
///
/// `LayerStack` provides a way to accumulate layers and apply them to a router in the
/// order they were added. Layers are applied via the `push()` method during configuration,
/// and then applied to the router via `apply()` during the build phase.
pub struct LayerStack {
    layers: Vec<LayerFn>,
}

impl LayerStack {
    pub fn new() -> Self {
        Self { layers: Vec::new() }
    }

    /// Add a layer to the stack.
    ///
    /// Layers are applied in FIFO order when `apply()` is called. Each layer will
    /// wrap the router after all adapters have been registered.
    pub fn push<L>(&mut self, layer: L)
    where
        L: Layer<Route> + Clone + Send + Sync + 'static,
        L::Service: Service<AxumRequest> + Clone + Send + Sync + 'static,
        <L::Service as Service<AxumRequest>>::Response: IntoResponse + 'static,
        <L::Service as Service<AxumRequest>>::Error: Into<Infallible> + 'static,
        <L::Service as Service<AxumRequest>>::Future: Send + 'static,
    {
        self.layers
            .push(Box::new(move |router| router.layer(layer.clone())));
    }

    /// Apply all accumulated layers to the given router in order.
    ///
    /// Layers are applied sequentially, with each layer wrapping the result of the previous one.
    pub fn apply(&self, mut router: Router) -> Router {
        for layer_fn in &self.layers {
            router = layer_fn(router);
        }

        router
    }

    pub fn clear(&mut self) {
        self.layers.clear();
    }
}

impl Default for LayerStack {
    fn default() -> Self {
        Self::new()
    }
}
