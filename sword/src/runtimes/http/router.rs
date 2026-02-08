use crate::adapters::http::{ControllerMeta, RouteRegistrar};
use crate::adapters::{AdapterKind, AdapterRegistry};
use axum::Router;
use std::any::TypeId;
use std::collections::HashMap;
use sword_core::layers::*;
use sword_core::{Config, State};

#[cfg(feature = "adapter-socketio")]
use super::socketio_config::{
    SocketIoParser, SocketIoServerConfig, SocketIoServerLayer,
};

pub struct HttpRouter {
    state: State,
    config: Config,
}

impl HttpRouter {
    pub fn new(state: State, config: Config) -> Self {
        Self { state, config }
    }

    #[cfg(feature = "adapter-socketio")]
    fn socketio_setup(
        &self,
    ) -> (
        Option<socketioxide::layer::SocketIoLayer>,
        SocketIoServerConfig,
    ) {
        let socketio_config = self.config.get_or_default::<SocketIoServerConfig>();

        let layer = socketio_config.enabled.then(|| {
            let (layer, io) = SocketIoServerLayer::new(&socketio_config);
            self.state.insert(io);
            layer
        });

        (layer, socketio_config)
    }

    #[cfg(feature = "adapter-socketio")]
    fn apply_socketio_layer(
        &self,
        mut router: Router<State>,
        layer: socketioxide::layer::SocketIoLayer,
        config: SocketIoServerConfig,
    ) -> Router<State> {
        use axum::{extract::Request, middleware::Next};

        router = router.layer(layer);

        router = router.layer(axum::middleware::from_fn(
            move |mut req: Request, next: Next| async move {
                req.extensions_mut().insert::<SocketIoParser>(config.parser);
                next.run(req).await
            },
        ));

        router
    }

    /// Build the complete HTTP router with all adapters and layers
    pub fn build(
        self,
        layers: LayerStack<State>,
        adapters: &AdapterRegistry,
    ) -> Router<State> {
        let mut router = Router::new();

        #[cfg(feature = "adapter-socketio")]
        let (socketio_layer, socketio_config) = self.socketio_setup();

        router = self.apply_adapters(router, &adapters.read());
        router = self.apply_http_specific_middlewares(router);

        #[cfg(feature = "adapter-socketio")]
        if let Some(layer) = socketio_layer {
            router = self.apply_socketio_layer(router, layer, socketio_config);
        }

        router = self.apply_runtime_agnostic_middlewares(router);
        router = layers.apply(router);

        router
    }

    /// Apply all adapters based on kind
    fn apply_adapters(
        &self,
        mut router: Router<State>,
        adapters: &HashMap<AdapterKind, Vec<TypeId>>,
    ) -> Router<State> {
        for (kind, adapters) in adapters.iter() {
            match kind {
                #[cfg(feature = "adapter-http-controllers")]
                AdapterKind::HttpController => {
                    router = self.apply_http_controllers(router, adapters);
                }
                #[cfg(feature = "adapter-socketio")]
                AdapterKind::SocketIo => {
                    self.apply_socketio_handlers(adapters);
                }
            }
        }

        router
    }

    fn apply_http_controllers(
        &self,
        mut router: Router<State>,
        controllers: &[TypeId],
    ) -> Router<State> {
        for controller_id in controllers {
            let mut controller_routes = inventory::iter::<RouteRegistrar>()
                .filter(|reg| &reg.controller_id == controller_id)
                .peekable();

            let controller_meta = controller_routes
                .peek()
                .map(|reg| ControllerMeta::from(*reg))
                .unwrap_or_else(|| {
                    eprintln!("ERROR: Controller with TypeId {controller_id:?} has no registered routes.");
                    eprintln!("This indicates a bug in the #[controller] macro implementation.");
                    panic!("No routes found for controller");
                });

            let mut controller_router = Router::new();

            for route in controller_routes {
                controller_router = controller_router
                    .route(route.path, (route.handler)(self.state.clone()));
            }

            controller_router = (controller_meta.apply_top_level_interceptors)(
                controller_router,
                self.state.clone(),
            );

            if controller_meta.controller_path == "/" {
                router = router.merge(controller_router);
            } else {
                router =
                    router.nest(controller_meta.controller_path, controller_router);
            }
        }

        router
    }

    /// Apply SocketIO handlers by calling their setup functions
    #[cfg(feature = "adapter-socketio")]
    fn apply_socketio_handlers(&self, handlers: &[TypeId]) {
        use crate::adapters::socketio::{
            HandlerRegistrar, SocketIoHandlerRegistrar,
        };

        for handler_id in handlers {
            let setup_fn = inventory::iter::<SocketIoHandlerRegistrar>()
                .find(|s| &s.handler_type_id == handler_id);

            if let Some(setup) = setup_fn {
                (setup.setup_fn)(&self.state);
            } else {
                let has_handlers = inventory::iter::<HandlerRegistrar>()
                    .any(|h| &h.adapter_type_id == handler_id);

                if has_handlers {
                    eprintln!(
                        "Warning: SocketIO adapter {:?} has handlers but no setup function. \
                            Did you forget #[on(\"connection\")] handler?",
                        handler_id
                    );
                }
            }
        }
    }

    /// Apply HTTP-specific layers
    ///
    /// These are applied BEFORE the SocketIO layer, so SocketIO requests bypass them.
    /// - RequestTimeout: Can interrupt long-lived SocketIO connections
    /// - RequestId: HTTP request tracking
    /// - CookieManager: Cookie handling
    fn apply_http_specific_middlewares(
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

    /// Apply runtime-agnostic layers
    ///
    /// These are applied AFTER the SocketIO layer, so they affect both REST and SocketIO.
    /// - CORS: Required for cross-origin SocketIO connections
    /// - Compression: Compresses REST responses and SocketIO handshake
    /// - ServeDir: Static files accessible to all
    fn apply_runtime_agnostic_middlewares(
        &self,
        mut router: Router<State>,
    ) -> Router<State> {
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
