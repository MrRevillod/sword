use std::convert::Infallible;

use axum::{
    extract::Request as AxumRequest,
    middleware::from_fn_with_state as mw,
    response::IntoResponse,
    routing::{Route, Router},
};

use crate::core::middlewares::*;

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
    /// modules, add middleware layers, and set up dependency injection.
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

    /// Register a module with the application builder.
    ///
    /// Can be used with any type that implements the `Module` trait. No matter if the module
    /// has controllers or not, this method will handle both cases.
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

    /// Adds a `tower::Layer` to the application builder.
    /// This method is equivalent to Axum's `Router::layer` method, allowing you to
    /// apply middleware layers to the application's router.
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

    /// Register a provider with the application's dependency injection container.
    ///
    /// This method can be used to add providers directly to the container, avoiding the need
    /// to create a full module when only a provider is needed.
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

        router = self.apply_controllers(router);
        router = self.apply_layers(router);
        router = self.apply_tower_layers(router);

        router = router.layer(mw(self.state.clone(), ContentTypeCheck::layer));

        let app_config = self.config.get::<ApplicationConfig>()
            .expect("Failed to get ApplicationConfig. Ensure it is present in the config file.");

        if let Some(prefix) = app_config.global_prefix {
            router = Router::new().nest(&prefix, router);
        }

        router
    }

    /// Build the `Application` instance with the configured options.
    /// This method ends the builder pattern and constructs the final `Application`
    /// instance ready to run.
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

    fn apply_layers(&self, router: Router) -> Router {
        let mut router = router;

        for layer_fn_applier in &self.layers {
            router = layer_fn_applier(router);
        }

        router
    }

    // Merge all the "controllers" routers into the main router
    // In fact, controllers are just functions that return a Router
    fn apply_controllers(&self, router: Router) -> Router {
        let mut router = router;

        for controller in &self.controllers {
            let controller = controller(self.state.clone());
            router = router.merge(controller);
        }

        router
    }

    fn apply_tower_layers(&self, router: Router) -> Router {
        let mut router = router;

        let middlewares_config = self
            .config
            .get::<MiddlewaresConfig>()
            .expect("Failed to get MiddlewaresConfig. Ensure it is present in the config file.");

        if middlewares_config.body_limit.enabled {
            router = router
                .layer(BodyLimitLayer::new(middlewares_config.body_limit.parsed));
        }

        if middlewares_config.request_timeout.enabled {
            let (timeout_service, response_mapper) =
                TimeoutLayer::new(middlewares_config.request_timeout.parsed);

            router = router.layer(timeout_service);
            router = router.layer(response_mapper);
        }

        if let Some(cors_config) = &middlewares_config.cors {
            router = router.layer(CorsLayer::new(cors_config))
        };

        if let Some(compression_config) = &middlewares_config.compression {
            if let Some(layer) =
                CompressionLayer::new(compression_config.compression.clone())
            {
                router = router.layer(layer);
            }
        }

        if let Some(serve_dir_config) = &middlewares_config.serve_dir {
            if serve_dir_config.enabled {
                let serve_dir = ServeDirMiddleware::new(serve_dir_config.clone());
                router =
                    router.nest_service(&serve_dir_config.router_path, serve_dir);
            }
        }

        #[cfg(feature = "cookies")]
        {
            router = router.layer(CookieManagerLayer::new());
        }

        router
    }
}

impl Default for ApplicationBuilder {
    fn default() -> Self {
        Self::new()
    }
}
