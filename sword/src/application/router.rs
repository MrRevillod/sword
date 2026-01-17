use crate::adapters::{AdapterKind, AdapterRegistry};
use axum::Router;
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

    pub fn build(
        self,
        adapters: &AdapterRegistry,
        layers: LayerStack<State>,
    ) -> Router<State> {
        #[cfg(feature = "adapter-socketio")]
        let socketio_layer: Option<socketioxide::layer::SocketIoLayer> = {
            use sword_layers::socketio::*;

            let socketio_config =
                self.config.get_or_default::<SocketIoServerConfig>();

            socketio_config.enabled.then(|| {
                let (layer, io) = SocketIoServerLayer::new(&socketio_config);
                self.state.insert(io);

                layer
            })
        };

        // Create router with state from the beginning
        // This allows handlers to use State<S> extractor
        let mut router = Router::new();

        // Register all adapters (REST controllers and SocketIO handlers)
        // SocketIO handlers can now access SocketIo from state
        let registered_types = adapters.registered_types();
        router =
            self.apply_adapters(router, &adapters.inner().read(), &registered_types);

        // Apply REST-only middlewares (internal to SocketIO layer)
        router = self.apply_rest_only_middlewares(router);

        // Apply SocketIO layer to router
        // This wraps the router but SocketIO requests bypass inner middlewares
        #[cfg(feature = "adapter-socketio")]
        {
            if let Some(socketio_layer) = socketio_layer {
                use axum::{extract::Request, middleware::Next};
                use sword_layers::socketio::*;

                let socketio_config =
                    self.config.get_or_default::<SocketIoServerConfig>();

                let parsed = socketio_config.parser;

                router = router.layer(socketio_layer);

                // Then apply parser middleware (outer - executes first)
                // This ensures extensions are set before SocketIO handshake
                router = router.layer(axum::middleware::from_fn(
                    move |mut req: Request, next: Next| async move {
                        req.extensions_mut().insert::<SocketIoParser>(parsed);
                        next.run(req).await
                    },
                ));
            }
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
        adapters: &[AdapterKind],
        registered_types: &[std::any::TypeId],
    ) -> Router<State> {
        // Apply SocketIO adapters (non-REST)
        for adapter_kind in adapters.iter() {
            match adapter_kind {
                AdapterKind::Rest(_) => {
                    // REST controllers are now handled via inventory
                    // This maintains backward compatibility but the actual routing
                    // is done through apply_auto_registered_routes
                }
                AdapterKind::SocketIo(setup_fn) => {
                    setup_fn(&self.state);
                }
                AdapterKind::Grpc => {
                    // Not implemented yet
                }
            }
        }

        // Build REST routes from inventory, filtering by registered TypeIds
        router = self.apply_auto_registered_routes(router, registered_types);

        router
    }

    /// Build routers from auto-registered routes via inventory
    ///
    /// This iterates all RouteRegistrar entries (submitted by #[get], #[post], etc.)
    /// and groups them by controller to build complete controller routers.
    ///
    /// If registered_types is not empty, only routes from those controllers are built.
    /// If registered_types is empty, all routes from inventory are built (full auto-registration).
    ///
    /// Note: This is called AFTER all providers/components are registered in State,
    /// so controllers can be built with their dependencies.
    fn apply_auto_registered_routes(
        &self,
        mut router: Router<State>,
        registered_types: &[std::any::TypeId],
    ) -> Router<State> {
        use crate::adapters::rest::RouteRegistrar;
        use axum::routing::MethodRouter;
        use std::collections::HashMap;

        // Group routes by (controller_name, controller_path)
        // Also store the interceptor function (should be the same for all routes of a controller)
        type ControllerKey = (&'static str, &'static str);
        type RouteInfo = (&'static str, fn(State) -> MethodRouter<State>);
        let mut controllers: HashMap<
            ControllerKey,
            (Vec<RouteInfo>, fn(Router<State>, State) -> Router<State>),
        > = HashMap::new();

        let use_all_controllers = registered_types.is_empty();

        for route_registrar in inventory::iter::<RouteRegistrar> {
            // Filter: only include routes from registered controllers
            // If no controllers registered manually, include all (full auto-registration)
            if !use_all_controllers
                && !registered_types.contains(&route_registrar.controller_type_id)
            {
                continue;
            }

            let key = (
                route_registrar.controller_name,
                route_registrar.controller_path,
            );
            let entry = controllers
                .entry(key)
                .or_insert_with(|| (Vec::new(), route_registrar.apply_interceptors));
            entry
                .0
                .push((route_registrar.route_path, route_registrar.build_handler));
        }

        // Build a router for each controller
        // Create routers with state so handlers can use State<S> extractor
        for ((_controller_name, controller_path), (routes, apply_interceptors_fn)) in
            controllers
        {
            let mut controller_router = Router::new();

            for (route_path, build_handler) in routes {
                // Execute the build function with State to create controller and handler
                // Handler is MethodRouter<State> - can use State<S> extractor
                let handler = build_handler(self.state.clone());
                controller_router = controller_router.route(route_path, handler);
            }

            // Apply controller-level interceptors (Router<State>, State) -> Router<State>
            controller_router =
                apply_interceptors_fn(controller_router, self.state.clone());

            // Nest the controller router under its base path
            if controller_path == "/" {
                router = router.merge(controller_router);
            } else {
                router = router.nest(controller_path, controller_router);
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
