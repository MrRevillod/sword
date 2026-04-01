use crate::controllers::web::{ControllerMeta, RouteRegistrar};
use crate::controllers::{Controller, ControllerRegistry};
use crate::engines::web::WebApplicationConfig;

use axum::{
    Router,
    {extract::Request, middleware::Next},
};

use std::any::TypeId;
use std::collections::{HashMap, HashSet};
use sword_core::{State, layers::*, sword_error};

#[cfg(feature = "socketio-controllers")]
use crate::controllers::socketio::{
    SocketIoParser, SocketIoServerConfig, SocketIoServerLayer,
};

pub struct WebRouter {
    state: State,
    web_config: WebApplicationConfig,

    #[cfg(feature = "socketio-controllers")]
    socketio_config: SocketIoServerConfig,
}

impl WebRouter {
    pub fn new(
        state: State,
        web_config: WebApplicationConfig,
        #[cfg(feature = "socketio-controllers")]
        socketio_config: SocketIoServerConfig,
    ) -> Self {
        Self {
            state,
            web_config,

            #[cfg(feature = "socketio-controllers")]
            socketio_config,
        }
    }

    #[cfg(feature = "socketio-controllers")]
    fn socketio_setup(
        &self,
    ) -> (
        Option<socketioxide::layer::SocketIoLayer>,
        SocketIoServerConfig,
    ) {
        let socketio_config = self.socketio_config.clone();

        let layer = socketio_config.enabled.then(|| {
            let (layer, io) = SocketIoServerLayer::new(&socketio_config);
            self.state.insert(io);
            layer
        });

        (layer, socketio_config)
    }

    #[cfg(feature = "socketio-controllers")]
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

    /// Build the complete HTTP router with all controllers and layers
    pub fn build(
        self,
        layers: LayerStack<State>,
        controllers: &ControllerRegistry,
    ) -> Router<State> {
        let mut router = Router::new();

        #[cfg(feature = "socketio-controllers")]
        let (socketio_layer, socketio_config) = self.socketio_setup();

        router = self.apply_controllers(router, &controllers.read());
        router = self.apply_web_layers(router);

        #[cfg(feature = "socketio-controllers")]
        if let Some(layer) = socketio_layer {
            router = self.apply_socketio_layer(router, layer, socketio_config);
        }

        router = layers.apply(router);

        router
    }

    /// Apply all controllers based on kind
    fn apply_controllers(
        &self,
        mut router: Router<State>,
        controllers: &HashMap<Controller, HashSet<TypeId>>,
    ) -> Router<State> {
        for (kind, controller_ids) in controllers.iter() {
            match kind {
                #[cfg(feature = "web-controllers")]
                Controller::Web => {
                    router = self.apply_http_controllers(router, controller_ids);
                }

                #[cfg(feature = "socketio-controllers")]
                Controller::SocketIo => {
                    self.apply_socketio_handlers(controller_ids);
                }

                #[cfg(feature = "grpc-controllers")]
                Controller::Grpc => {}
            }
        }

        router
    }

    fn apply_http_controllers(
        &self,
        mut router: Router<State>,
        controllers: &HashSet<TypeId>,
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
    #[cfg(feature = "socketio-controllers")]
    fn apply_socketio_handlers(&self, handlers: &HashSet<TypeId>) {
        use crate::controllers::socketio::{
            HandlerRegistrar, SocketIoHandlerRegistrar,
        };

        for handler_id in handlers {
            let setup_fn = inventory::iter::<SocketIoHandlerRegistrar>()
                .find(|s| &s.handler_type_id == handler_id);

            if let Some(setup) = setup_fn {
                (setup.setup_fn)(&self.state);
            } else {
                let has_handlers = inventory::iter::<HandlerRegistrar>()
                    .any(|h| &h.controller_type_id == handler_id);

                if has_handlers {
                    sword_error! {
                        title: "Controller has handlers but no setup function",
                        reason: "SocketIoHandlerRegistrar is missing for controller",
                        context: {
                            "handler_id" => format!("{handler_id:?}"),
                            "source" => "WebRouter::apply_socketio_handlers",
                        },
                        hints: ["Verify #[controller(kind = Controller::SocketIo, namespace = \"...\")] and #[on(...)] annotations are applied correctly"],
                    };
                }
            }
        }
    }

    /// Apply mandatory web layers.
    ///
    /// These are applied BEFORE the SocketIO layer, so SocketIO traffic bypasses
    /// HTTP controller timeout semantics.
    fn apply_web_layers(&self, mut router: Router<State>) -> Router<State> {
        let server_config = self.web_config.clone();
        let body_limit_config = server_config.body_limit;

        if server_config.request_timeout.enabled {
            let timeout_service: TimeoutLayer =
                server_config.request_timeout.clone().into();
            let response_mapper = RequestTimeoutResponseLayer::new();

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
