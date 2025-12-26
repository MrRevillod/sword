use crate::core::application::layer_stack::LayerStack;
use crate::core::{AdapterKind, AdapterRegistry, Config, State};
use crate::web::MiddlewaresConfig;

use axum::Router;
#[cfg(feature = "socketio")]
use socketioxide::layer::SocketIoLayer;
use sword_layers::prelude::*;

pub(super) struct InternalRouter {
    state: State,
    config: Config,
}

impl InternalRouter {
    pub fn new(state: State, config: Config) -> Self {
        Self { state, config }
    }

    pub fn build(self, adapters: &AdapterRegistry, layers: LayerStack) -> Router {
        #[cfg(feature = "socketio")]
        let socketio_layer: Option<SocketIoLayer> = {
            use sword_layers::socketio::*;

            let socketio_config =
                self.config.get_or_default::<SocketIoServerConfig>();

            socketio_config.enabled.then(|| {
                let (layer, io) = SocketIoServerLayer::new(socketio_config);
                self.state.insert(io);

                layer
            })
        };

        // Create router with state (now containing SocketIo if enabled)
        let mut router = Router::new().with_state(self.state.clone());

        // Register all adapters (REST controllers and SocketIO handlers)
        // SocketIO handlers can now access SocketIo from state
        router = self.apply_adapters(router, &adapters.inner().read());

        // Apply REST-only middlewares (internal to SocketIO layer)
        router = self.apply_rest_only_middlewares(router);

        // Apply SocketIO layer to router
        // This wraps the router but SocketIO requests bypass inner middlewares
        #[cfg(feature = "socketio")]
        {
            if let Some(socketio_layer) = socketio_layer {
                router = router.layer(socketio_layer);
            }
        }

        // Apply shared middlewares (external to SocketIO layer - affects both)
        // These wrap everything including SocketIO
        router = self.apply_shared_middlewares(router);

        // Apply custom user middlewares from layer stack
        router = layers.apply(router);

        router
    }

    fn apply_adapters(
        &self,
        mut router: Router,
        adapters: &[AdapterKind],
    ) -> Router {
        for adapter_kind in adapters.iter() {
            match adapter_kind {
                AdapterKind::Rest(builder) => {
                    let rest_router = builder(self.state.clone());
                    router = router.merge(rest_router);
                }
                AdapterKind::SocketIo(setup_fn) => {
                    setup_fn(&self.state);
                }
                AdapterKind::Grpc => {
                    // Not implemented yet
                }
            }
        }

        router
    }

    /// Apply middlewares that should ONLY affect REST routes
    ///
    /// These are applied BEFORE the SocketIO layer, so SocketIO requests
    /// bypass them completely.
    ///
    /// - RequestTimeout: Can interrupt long-lived SocketIO connections
    /// - BodyLimit: REST-specific (SocketIO uses max_payload config)
    fn apply_rest_only_middlewares(&self, mut router: Router) -> Router {
        let middlewares_config = self.config.get_or_default::<MiddlewaresConfig>();

        if middlewares_config.request_timeout.enabled {
            let (timeout_service, response_mapper) =
                RequestTimeoutLayer::new(&middlewares_config.request_timeout);

            router = router.layer(timeout_service);
            router = router.layer(response_mapper);
        }

        // Request ID and Cookie Manager - Apply early for REST
        // These will also be available during SocketIO handshake
        router = router.layer(RequestIdLayer::new());
        router = router.layer(CookieManagerLayer::new());

        router
    }

    /// Apply middlewares that affect BOTH REST and SocketIO
    ///
    /// These are applied AFTER the SocketIO layer, so they wrap everything.
    /// SocketIO handshake requests pass through these middlewares.
    ///
    /// - CORS: Required for cross-origin SocketIO connections
    /// - Compression: Compresses REST responses and SocketIO handshake (but not WebSocket frames)
    /// - ServeDir: Static files accessible to all
    fn apply_shared_middlewares(&self, mut router: Router) -> Router {
        let middlewares_config = self.config.get_or_default::<MiddlewaresConfig>();

        if middlewares_config.cors.enabled {
            router = router.layer(CorsLayer::new(&middlewares_config.cors));
        }

        if middlewares_config.compression.enabled {
            router =
                router.layer(CompressionLayer::new(&middlewares_config.compression));
        }

        let serve_dir_config = self.config.get_or_default::<ServeDirConfig>();

        if serve_dir_config.enabled {
            let serve_dir = ServeDirLayer::new(&serve_dir_config);
            router = router.nest_service(&serve_dir_config.router_path, serve_dir);
        }

        router
    }
}
