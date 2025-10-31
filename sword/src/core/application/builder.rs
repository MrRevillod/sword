use std::{convert::Infallible, time::Duration};

use axum::{
    extract::Request as AxumRequest,
    middleware::from_fn_with_state as mw_with_state,
    response::IntoResponse,
    routing::{Route, Router},
};

use crate::web::{CorsConfig, CorsLayer};

use tower::{Layer, Service};
use tower_http::{limit::RequestBodyLimitLayer, timeout::TimeoutLayer};

#[cfg(feature = "cookies")]
use tower_cookies::CookieManagerLayer;

use crate::{
    core::*,
    web::{ContentTypeCheck, Controller, MiddlewareRegistrar, ResponsePrettifier},
};

pub struct ApplicationBuilder {
    router: Router,
    state: State,
    config: Config,
    controllers: Vec<fn(State) -> Router>,
    container: DependencyContainer,
    layers: Vec<Box<dyn Fn(Router) -> Router + Send + Sync>>,
}

impl ApplicationBuilder {
    // Builder for constructing a Sword application with various configuration options.
    //
    // `ApplicationBuilder` provides a fluent interface for configuring a Sword application
    // before building the final `Application` instance. It allows you to register
    // controllers, add middleware layers, configure shared state, and set up dependency injection.
    //
    // ### Example
    //
    // ```rust,ignore
    // use sword::prelude::*;
    //
    // let app = Application::builder()
    //     .build();
    // ```
    pub fn new() -> Self {
        let state = State::new();
        let config = Config::new().expect("Configuration loading error");

        state
            .insert(config.clone())
            .expect("Failed to insert Config into State");

        for ConfigRegistrar { register } in inventory::iter::<ConfigRegistrar> {
            register(&config, &state).expect("Failed to register config type");
        }

        let router = Router::new().with_state(state.clone());

        Self {
            router,
            state,
            config,
            controllers: Vec::new(),
            container: DependencyContainer::builder(),
            layers: Vec::new(),
        }
    }

    pub fn with_module<M>(mut self) -> Self
    where
        M: Module,
    {
        futures::executor::block_on(async {
            M::register_providers(&self.config, &mut self.container).await;
        });

        M::register_components(&mut self.container);

        if M::is_controller_module() {
            self.controllers.push(M::Controller::router);
        }

        Self {
            router: self.router,
            state: self.state,
            config: self.config,
            controllers: self.controllers,
            container: self.container,
            layers: self.layers,
        }
    }

    pub fn with_layer<L>(mut self, layer: L) -> Self
    where
        L: Layer<Route> + Clone + Send + Sync + 'static,
        L::Service: Service<AxumRequest> + Clone + Send + Sync + 'static,
        <L::Service as Service<AxumRequest>>::Response: IntoResponse + 'static,
        <L::Service as Service<AxumRequest>>::Error: Into<Infallible> + 'static,
        <L::Service as Service<AxumRequest>>::Future: Send + 'static,
    {
        self.layers
            .push(Box::new(move |router| router.layer(layer.clone())));

        Self {
            router: self.router,
            state: self.state,
            config: self.config,
            container: self.container,
            controllers: self.controllers,
            layers: self.layers,
        }
    }

    pub fn config<T>(&self) -> Result<T, ConfigError>
    where
        T: ConfigItem,
    {
        self.config.get::<T>()
    }

    pub fn build(self) -> Application {
        let mut router = self.router.clone();
        let app_config = self.config.get::<ApplicationConfig>().unwrap();

        self.container
            .build_all(&self.state)
            .unwrap_or_else(|e| panic!("Failed to build dependencies: {e}"));

        for MiddlewareRegistrar { register_fn } in
            inventory::iter::<MiddlewareRegistrar>
        {
            (register_fn)(&self.state).expect("Failed to register middleware");
        }

        for controller_router_fn in self.controllers {
            let controller = controller_router_fn(self.state.clone());
            router = router.merge(controller);
        }

        for layer_fn in self.layers {
            router = layer_fn(router);
        }

        router = router
            .layer(mw_with_state(self.state.clone(), ContentTypeCheck::layer))
            .layer(RequestBodyLimitLayer::new(app_config.body_limit.parsed));

        if let Ok(cors_config) = self.config.get::<CorsConfig>() {
            router = router.layer(CorsLayer::new(&cors_config))
        };

        if let Some(timeout_secs) = app_config.request_timeout_seconds {
            router =
                router.layer(TimeoutLayer::new(Duration::from_secs(timeout_secs)));
        }

        #[cfg(feature = "cookies")]
        {
            router = router.layer(CookieManagerLayer::new());
        }

        router = router
            .layer(mw_with_state(self.state.clone(), ResponsePrettifier::layer));

        if let Some(prefix) = app_config.global_prefix {
            router = Router::new().nest(&prefix, router);
        }

        Application::new(router, self.config)
    }
}

impl Default for ApplicationBuilder {
    fn default() -> Self {
        Self::new()
    }
}
