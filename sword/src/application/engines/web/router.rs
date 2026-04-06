#[cfg(feature = "socketio-controllers")]
use crate::controllers::socketio::{
    HandlerRegistrar, SocketIoHandlerRegistrar, SocketIoLayer, SocketIoParser,
    SocketIoServerConfig, SocketIoServerLayer,
};
use crate::controllers::web::{RouteRegistrar, WebControllerRegistrar};
use crate::controllers::{Controller, ControllerIds, ControllerMap, ControllerRegistry};
use crate::{application::ApplicationConfig, engines::web::WebApplicationConfig};

use crate::web::JsonResponse;
use axum::Router;
use axum::{extract::Request, middleware::Next};
use std::any::TypeId;
use std::collections::HashMap;
#[cfg(feature = "socketio-controllers")]
use std::collections::HashSet;
use sword_core::{Config, State, sword_error};
use sword_layers::{
    body_limit::{BodyLimitLayer, BodyLimitValue},
    layer_stack::LayerStack,
    not_found::NotFoundLayer,
    timeout::{RequestTimeoutResponseLayer, TimeoutLayer},
};

pub(crate) struct WebRouter<'a> {
    pub state: State,
    pub config: &'a Config,
    pub layer_stack: LayerStack<State>,
    pub controller_registry: &'a ControllerRegistry,
}

impl<'a> WebRouter<'a> {
    #[cfg(feature = "socketio-controllers")]
    fn socketio_setup(
        state: &State,
        socketio_config: &SocketIoServerConfig,
    ) -> (Option<SocketIoLayer>, SocketIoServerConfig) {
        let socketio_config = socketio_config.clone();

        let layer = socketio_config.enabled.then(|| {
            let (layer, io) = SocketIoServerLayer::new(&socketio_config);
            state.insert(io);
            layer
        });

        (layer, socketio_config)
    }

    #[cfg(feature = "socketio-controllers")]
    fn apply_socketio_layer(
        mut router: Router<State>,
        layer: SocketIoLayer,
        config: SocketIoServerConfig,
    ) -> Router<State> {
        router = router.layer(layer);

        router = router.layer(axum::middleware::from_fn(
            move |mut req: Request, next: Next| async move {
                req.extensions_mut().insert::<SocketIoParser>(config.parser);
                next.run(req).await
            },
        ));

        router
    }

    /// Build the complete HTTP router with all framework controllers and layers.
    pub(crate) fn build(self) -> Router<State> {
        let app_config = self.config.get_or_default::<ApplicationConfig>();
        let web_config = app_config.web.clone();

        #[cfg(feature = "socketio-controllers")]
        let socketio_config = self.config.get_or_default::<SocketIoServerConfig>();

        let mut router = Router::new();

        #[cfg(feature = "socketio-controllers")]
        let (socketio_layer, socketio_config) = Self::socketio_setup(&self.state, &socketio_config);

        router = Self::apply_controllers(&self.state, router, &self.controller_registry.read());
        router = Self::apply_web_layers(router, &web_config);

        #[cfg(feature = "socketio-controllers")]
        if let Some(layer) = socketio_layer {
            router = Self::apply_socketio_layer(router, layer, socketio_config);
        }

        router = self.layer_stack.apply(router);

        if let Some(prefix) = &web_config.web_router_prefix {
            router = Router::new().nest(prefix, router);
        }

        router = router.route(
            "/health",
            axum::routing::get(|| async { JsonResponse::Ok().message("healthy") }),
        );

        router.layer(NotFoundLayer::new())
    }

    /// Apply all controllers based on kind
    fn apply_controllers(
        state: &State,
        mut router: Router<State>,
        controllers: &ControllerMap,
    ) -> Router<State> {
        for (kind, ids) in controllers.iter() {
            match kind {
                #[cfg(feature = "web-controllers")]
                Controller::Web => {
                    router = Self::apply_web_controllers(state, router, ids);
                }
                #[cfg(feature = "socketio-controllers")]
                Controller::SocketIo => {
                    Self::apply_socketio_controllers(state, ids);
                }
                #[cfg(feature = "grpc-controllers")]
                Controller::Grpc => {}
            }
        }

        router
    }

    fn apply_web_controllers(
        state: &State,
        mut router: Router<State>,
        controllers: &ControllerIds,
    ) -> Router<State> {
        let controller_registrars: HashMap<TypeId, &WebControllerRegistrar> =
            inventory::iter::<WebControllerRegistrar>()
                .map(|reg| (reg.controller_id, reg))
                .collect();

        let mut routes_by_controller: HashMap<TypeId, Vec<&RouteRegistrar>> = HashMap::new();

        for route in inventory::iter::<RouteRegistrar>() {
            routes_by_controller
                .entry(route.controller_id)
                .or_default()
                .push(route);
        }

        for controller_id in controllers {
            let controller_registrar = controller_registrars
                .get(controller_id)
                .copied()
                .unwrap_or_else(|| {
                    sword_error! {
                        title: "Controller metadata not found",
                        reason: "No WebControllerRegistrar entry was found for controller",
                        context: {
                            "controller_id" => format!("{controller_id:?}"),
                            "source" => "WebRouter::apply_http_controllers",
                        },
                        hints: ["This usually indicates a controller macro expansion issue"],
                    }
                });

            (controller_registrar.build)(state);

            let controller_routes = routes_by_controller
                .get(controller_id)
                .cloned()
                .unwrap_or_default();

            if controller_routes.is_empty() {
                sword_error! {
                    title: "Controller has no registered routes",
                    reason: "No RouteRegistrar entries were found for controller",
                    context: {
                        "controller_id" => format!("{controller_id:?}"),
                        "source" => "WebRouter::apply_http_controllers",
                    },
                    hints: ["This usually indicates a controller macro expansion issue"],
                }
            }

            let mut controller_router = Router::new();

            for route in controller_routes {
                controller_router =
                    controller_router.route(route.path, (route.handler)(state.clone()));
            }

            if controller_registrar.controller_path == "/" {
                router = router.merge(controller_router);
            } else {
                router = router.nest(controller_registrar.controller_path, controller_router);
            }
        }

        router
    }

    /// Apply SocketIO handlers by calling their setup functions
    #[cfg(feature = "socketio-controllers")]
    fn apply_socketio_controllers(state: &State, handlers: &ControllerIds) {
        let setup_fns: HashMap<TypeId, &SocketIoHandlerRegistrar> =
            inventory::iter::<SocketIoHandlerRegistrar>()
                .map(|setup| (setup.handler_type_id, setup))
                .collect();

        let handler_controllers: HashSet<TypeId> = inventory::iter::<HandlerRegistrar>()
            .map(|handler| handler.controller_type_id)
            .collect();

        for handler_id in handlers {
            if let Some(setup) = setup_fns.get(handler_id).copied() {
                (setup.setup_fn)(state);
            } else {
                let has_handlers = handler_controllers.contains(handler_id);

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
    fn apply_web_layers(
        mut router: Router<State>,
        web_config: &WebApplicationConfig,
    ) -> Router<State> {
        let body_limit_config = web_config.body_limit.clone();

        router = router.layer(BodyLimitLayer::new(&body_limit_config));

        if web_config.request_timeout.enabled {
            let timeout_service: TimeoutLayer = web_config.request_timeout.clone().into();

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
