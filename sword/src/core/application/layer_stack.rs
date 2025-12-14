use axum::{
    Router, extract::Request as AxumRequest, response::IntoResponse, routing::Route,
};
use std::convert::Infallible;
use tower::{Layer, Service};

pub(crate) type LayerFn = Box<dyn Fn(Router) -> Router + Send + Sync>;

/// A stack for managing and applying middleware layers to a router.
///
/// `LayerStack` provides a fluent interface for accumulating layers and applying them
/// to a router in the order they were added.
pub struct LayerStack {
    layers: Vec<LayerFn>,
}

impl LayerStack {
    pub fn new() -> Self {
        Self { layers: Vec::new() }
    }

    /// Add a layer to the stack.
    /// Layers are applied in the order they are added when `apply()` is called.
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

    pub fn apply(&self, mut router: Router) -> Router {
        for layer_fn in &self.layers {
            router = layer_fn(router);
        }

        router
    }
}

impl Default for LayerStack {
    fn default() -> Self {
        Self::new()
    }
}
