use crate::adapters::controllers::{ControllerMeta, RouteRegistrar};
use crate::adapters::{AdapterKind, AdapterRegistry};

use axum::{
    Router,
    {extract::Request, middleware::Next},
};

use std::any::TypeId;
use std::collections::HashMap;
use sword_core::layers::*;
use sword_core::{Config, StartupPhase, State, sword_error};

use super::WebRuntimeConfig;

#[cfg(feature = "web-adapter-socketio")]
use super::socketio_config::{
    SocketIoParser, SocketIoServerConfig, SocketIoServerLayer,
};

pub struct WebRouter {
    state: State,
    config: Config,
}

impl WebRouter {
    pub fn new(state: State, config: Config) -> Self {
        Self { state, config }
    }

    #[cfg(feature = "web-adapter-socketio")]
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

    #[cfg(feature = "web-adapter-socketio")]
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

        #[cfg(feature = "web-adapter-socketio")]
        let (socketio_layer, socketio_config) = self.socketio_setup();

        router = self.apply_adapters(router, &adapters.read());
        router = self.apply_web_mandatory_layers(router);

        #[cfg(feature = "web-adapter-socketio")]
        if let Some(layer) = socketio_layer {
            router = self.apply_socketio_layer(router, layer, socketio_config);
        }

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
                #[cfg(feature = "web-adapter-controllers")]
                AdapterKind::Controllers => {
                    router = self.apply_http_controllers(router, adapters);
                }

                #[cfg(feature = "web-adapter-socketio")]
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
                    sword_error! {
                        phase: StartupPhase::HttpAdapter,
                        title: "Controller has no registered routes",
                        reason: "No RouteRegistrar entries were found for controller",
                        context: {
                            "controller_id" => format!("{controller_id:?}"),
                            "source" => "WebRouter::apply_http_controllers",
                        },
                        hints: ["This usually indicates a controller macro expansion issue"],
                    }
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
    #[cfg(feature = "web-adapter-socketio")]
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
                    sword_error! {
                        phase: StartupPhase::SocketIoAdapter,
                        title: "Adapter has handlers but no setup function",
                        reason: "SocketIoHandlerRegistrar is missing for adapter",
                        context: {
                            "handler_id" => format!("{handler_id:?}"),
                            "source" => "WebRouter::apply_socketio_handlers",
                        },
                        hints: ["Verify #[socketio_adapter] and #[on(...)] annotations are applied correctly"],
                    };
                }
            }
        }
    }

    /// Apply mandatory web-runtime layers.
    ///
    /// These are applied BEFORE the SocketIO layer, so SocketIO traffic bypasses
    /// HTTP controller timeout semantics.
    fn apply_web_mandatory_layers(
        &self,
        mut router: Router<State>,
    ) -> Router<State> {
        let server_config = self.config.get_or_default::<WebRuntimeConfig>();
        let body_limit_config = server_config.body_limit;

        if server_config.request_timeout.enabled {
            let (timeout_service, response_mapper) =
                RequestTimeoutLayer::new(&server_config.request_timeout);

            router = router.layer(timeout_service);
            router = router.layer(response_mapper);
        }

        router = router.layer(axum::middleware::from_fn(
            move |mut req: Request, next: Next| async move {
                req.extensions_mut()
                    .insert(BodyLimitValue(body_limit_config.max_size.parsed));

                next.run(req).await
            },
        ));

        router
    }
}
