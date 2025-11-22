use std::convert::Infallible;

use axum::{
    extract::Request as AxumRequest,
    middleware::from_fn_with_state as mw_with_state,
    response::IntoResponse,
    routing::{Route, Router},
};

use crate::core::middlewares::{
    BodyLimitLayer, LimitsMiddlewareConfig, TimeoutLayer,
};

use tower::{Layer, Service};

#[cfg(feature = "cookies")]
use tower_cookies::CookieManagerLayer;

use crate::{
    core::*,
    web::{Controller, MiddlewareRegistrar},
};

pub struct ApplicationBuilder {
    pub config: Config,
    router: Router,
    state: State,
    controllers: Vec<fn(State) -> Router>,
    container: DependencyContainer,
    layers: Vec<Box<dyn Fn(Router) -> Router + Send + Sync>>,
}

impl ApplicationBuilder {
    /// Builder for constructing a Sword application with various configuration options.
    ///
    /// `ApplicationBuilder` provides a fluent interface for configuring a Sword application
    /// before building the final `Application` instance. It allows you to register
    /// controllers, add middleware layers, configure shared state, and set up dependency injection.
    pub fn new() -> Self {
        let state = State::new();
        let config = Config::new().expect("Configuration loading error");

        state.insert(config.clone());

        for ConfigRegistrar { register } in inventory::iter::<ConfigRegistrar> {
            register(&config, &state).expect("Failed to register config type");
        }

        let router = Router::new().with_state(state.clone());

        Self {
            router,
            state,
            config,
            controllers: Vec::new(),
            container: DependencyContainer::new(),
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

    pub fn with_provider<T>(mut self, provider: T) -> Self
    where
        T: Provider,
    {
        self.container.register_provider(provider);

        Self {
            router: self.router,
            state: self.state,
            config: self.config,
            container: self.container,
            controllers: self.controllers,
            layers: self.layers,
        }
    }

    fn build_router(&self) -> Router {
        let mut router = self.router.clone();

        // Merge all the "controllers" routers into the main router
        // In fact, controllers are just functions that return a Router
        for controller in &self.controllers {
            let controller = controller(self.state.clone());
            router = router.merge(controller);
        }

        // Apply all the layers setted via `with_layer` method
        for layer_fn_applier in &self.layers {
            router = layer_fn_applier(router);
        }

        router =
            router.layer(mw_with_state(self.state.clone(), ContentTypeCheck::layer));

        let app_config = self.config.get::<ApplicationConfig>().unwrap();
        let limits_config = self.config.get::<LimitsMiddlewareConfig>().unwrap();

        router = router.layer(BodyLimitLayer::new(limits_config.body.parsed));

        if let Some(TimeoutLimit { parsed, .. }) = limits_config.request_timeout {
            let (timeout_service, response_mapper) = TimeoutLayer::new(parsed);

            router = router.layer(timeout_service);
            router = router.layer(response_mapper);
        }

        if let Ok(cors_config) = self.config.get::<CorsConfig>() {
            router = router.layer(CorsLayer::new(&cors_config))
        };

        #[cfg(feature = "cookies")]
        {
            router = router.layer(CookieManagerLayer::new());
        }

        if let Some(prefix) = app_config.global_prefix {
            router = Router::new().nest(&prefix, router);
        }

        router
    }

    pub fn build(mut self) -> Application {
        self.container
            .build_all(&self.state)
            .expect("Failed to build dependency injection container");

        for MiddlewareRegistrar { register_fn } in
            inventory::iter::<MiddlewareRegistrar>
        {
            (register_fn)(&self.state).expect("Failed to register middleware");
        }

        let router = self.build_router();

        self.layers.clear();
        self.controllers.clear();
        self.container.clear();

        Application::new(router, self.config)
    }
}

impl Default for ApplicationBuilder {
    fn default() -> Self {
        Self::new()
    }
}
