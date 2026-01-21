use std::any::TypeId;
use std::collections::HashMap;

use crate::adapters::rest::{ControllerInfo, RouteRegistrar};
use crate::adapters::{AdapterKind, AdapterRegistry};
use axum::Router;
use socketioxide::layer::SocketIoLayer;
use sword_core::layers::*;
use sword_core::{Config, State};

pub(super) struct InternalRouter {
    state: State,
    config: Config,
}

impl InternalRouter {
    pub fn new(state: State, config: Config) -> Self {
        Self { state, config }
    }

    #[cfg(feature = "adapter-socketio")]
    pub fn socketio_setup(&self) -> (Option<SocketIoLayer>, SocketIoServerConfig) {
        let socketio_config = self.config.get_or_default::<SocketIoServerConfig>();

        let layer = socketio_config.enabled.then(|| {
            let (layer, io) = SocketIoServerLayer::new(&socketio_config);
            self.state.insert(io);

            layer
        });

        (layer, socketio_config)
    }

    #[cfg(feature = "adapter-socketio")]
    pub fn apply_socketio_layer(
        &self,
        mut router: Router<State>,
        layer: SocketIoLayer,
        config: SocketIoServerConfig,
    ) -> Router<State> {
        use axum::{extract::Request, middleware::Next};
        use sword_layers::socketio::*;

        router = router.layer(layer);

        // Apply parser middleware (outer - executes first)
        // This ensures extensions are set before SocketIO handshake
        router = router.layer(axum::middleware::from_fn(
            move |mut req: Request, next: Next| async move {
                req.extensions_mut().insert::<SocketIoParser>(config.parser);
                next.run(req).await
            },
        ));

        router
    }

    pub fn build(
        self,
        layers: LayerStack<State>,
        adapters: &AdapterRegistry,
    ) -> Router<State> {
        let mut router = Router::new();

        #[cfg(feature = "adapter-socketio")]
        let (socketio_layer, socketio_config) = self.socketio_setup();

        router = self.apply_adapters(router, &*adapters.read());

        // Apply REST-only middlewares (internal to SocketIO layer)
        router = self.apply_rest_only_middlewares(router);

        // Apply SocketIO layer to router
        // This wraps the router but SocketIO requests bypass inner middlewares
        #[cfg(feature = "adapter-socketio")]
        if let Some(layer) = socketio_layer {
            router = self.apply_socketio_layer(router, layer, socketio_config);
        }

        // Apply shared middlewares (external to SocketIO layer - affects both)
        router = self.apply_shared_middlewares(router);

        // Apply custom user middlewares from layer stack
        router = layers.apply(router);

        router
    }

    fn apply_adapters(
        &self,
        mut router: Router<State>,
        adapters: &HashMap<AdapterKind, Vec<TypeId>>,
    ) -> Router<State> {
        for (kind, adapters) in adapters.iter() {
            match kind {
                AdapterKind::Http => {
                    router = self.apply_http_adapters(router, adapters);
                }
                AdapterKind::SocketIo => {
                    self.apply_socketio_adapters(adapters);
                }
            }
        }

        router
    }

    fn apply_http_adapters(
        &self,
        mut router: Router<State>,
        adapters: &[TypeId],
    ) -> Router<State> {
        for adapter_id in adapters {
            let mut adapter_routes = inventory::iter::<RouteRegistrar>()
                .filter(|reg| &reg.controller_type_id == adapter_id)
                .peekable();

            let info = adapter_routes
                .peek()
                .map(|reg| ControllerInfo::from(*reg))
                .unwrap_or_else(|| {
                    eprintln!("ERROR: Adapter with TypeId {adapter_id:?} has no registered routes.");
                    eprintln!("This indicates a bug in the #[controller] macro implementation.");
                    panic!("No routes found for adapter");
                });

            let mut controller_router = Router::new();

            for route in adapter_routes {
                controller_router = controller_router
                    .route(route.route_path, (route.handler)(self.state.clone()));
            }

            controller_router = (info.apply_controller_level_interceptors)(
                controller_router,
                self.state.clone(),
            );

            if info.controller_path == "/" {
                router = router.merge(controller_router);
            } else {
                router = router.nest(info.controller_path, controller_router);
            }
        }

        router
    }

    #[cfg(feature = "adapter-socketio")]
    fn apply_socketio_adapters(&self, adapters: &[TypeId]) {
        use crate::adapters::socketio::{HandlerRegistrar, SocketIoSetupFn};

        for adapter_id in adapters {
            let setup_fn = inventory::iter::<SocketIoSetupFn>()
                .find(|s| &s.adapter_type_id == adapter_id);

            if let Some(setup) = setup_fn {
                (setup.setup)(&self.state);
            } else {
                let has_handlers = inventory::iter::<HandlerRegistrar>()
                    .any(|h| &h.adapter_type_id == adapter_id);

                if has_handlers {
                    eprintln!(
                        "Warning: SocketIO adapter {:?} has handlers but no setup function. \
                            Did you forget #[on(\"connection\")] handler?",
                        adapter_id
                    );
                }
            }
        }
    }

    /// Apply middlewares that should ONLY affect REST routes
    ///
    /// These are applied BEFORE the SocketIO layer, so SocketIO requests
    /// bypass them completely.
    ///
    /// - RequestTimeout: Can interrupt long-lived SocketIO connections
    /// - BodyLimit: REST-specific (SocketIO uses max_payload config)
    fn apply_rest_only_middlewares(
        &self,
        mut router: Router<State>,
    ) -> Router<State> {
        let middlewares_config = self.config.get_or_default::<MiddlewaresConfig>();

        if middlewares_config.request_timeout.enabled {
            let (timeout_service, response_mapper) =
                RequestTimeoutLayer::new(&middlewares_config.request_timeout);

            router = router.layer(timeout_service);
            router = router.layer(response_mapper);
        }

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
    fn apply_shared_middlewares(&self, mut router: Router<State>) -> Router<State> {
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
