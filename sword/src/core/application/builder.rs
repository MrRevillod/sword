use super::layer_stack::LayerStack;
use crate::core::__internal::ConfigRegistrar;
use crate::core::*;
use crate::web::__internal::MiddlewareRegistrar;
use crate::web::MiddlewaresConfig;

use axum::{
    extract::Request as AxumRequest,
    response::IntoResponse,
    routing::{Route, Router},
};

use std::convert::Infallible;
use sword_layers::prelude::*;
use tower::{Layer, Service};

pub struct ApplicationBuilder {
    state: State,
    container: DependencyContainer,
    adapter_registry: AdapterRegistry,
    layer_stack: LayerStack,
    pub config: Config,
}

impl ApplicationBuilder {
    /// Builder for constructing a Sword application with various configuration options.
    ///
    /// `ApplicationBuilder` provides a fluent interface for configuring a Sword application
    /// before building the final `Application` instance. It allows you to register
    /// modules, add middleware layers, and set up dependency injection.
    ///
    /// The builder follows this configuration pattern:
    /// 1. Create with `Application::builder()`
    /// 2. Register modules with `with_module::<M>()`
    /// 3. Optionally add custom layers with `with_layer()`
    /// 4. Optionally register providers directly with `with_provider()`
    /// 5. Build the application with `build()`
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let app = Application::builder()
    ///     .with_module::<UsersModule>()
    ///     .with_module::<ProductsModule>()
    ///     .with_layer(custom_middleware)
    ///     .build();
    ///
    /// app.run().await;
    /// ```
    pub fn new() -> Self {
        let state = State::new();
        let config = Config::new().expect("Configuration loading error");

        state.insert(config.clone());

        for ConfigRegistrar { register } in inventory::iter::<ConfigRegistrar> {
            register(&config, &state).expect("Failed to register config type");
        }

        Self {
            state,
            config,
            container: DependencyContainer::new(),
            adapter_registry: AdapterRegistry::new(),
            layer_stack: LayerStack::new(),
        }
    }

    /// Register a module with the application builder.
    /// Can be used with any type that implements the `Module` trait.
    pub fn with_module<M>(self) -> Self
    where
        M: Module,
    {
        futures::executor::block_on(async {
            M::register_providers(&self.config, self.container.provider_registry())
                .await;
        });

        M::register_components(self.container.component_registry());
        M::register_adapters(&self.adapter_registry);

        self
    }

    /// Adds a `tower::Layer` to the application builder.
    ///
    /// This method is equivalent to Axum's `Router::layer` method, allowing you to
    /// apply middleware layers to the application's router. Layers are applied in the
    /// order they are added, after gateways are registered but before Sword's built-in
    /// middleware layers (CORS, compression, request timeout, etc.).
    pub fn with_layer<L>(mut self, layer: L) -> Self
    where
        L: Layer<Route> + Clone + Send + Sync + 'static,
        L::Service: Service<AxumRequest> + Clone + Send + Sync + 'static,
        <L::Service as Service<AxumRequest>>::Response: IntoResponse + 'static,
        <L::Service as Service<AxumRequest>>::Error: Into<Infallible> + 'static,
        <L::Service as Service<AxumRequest>>::Future: Send + 'static,
    {
        self.layer_stack.push(layer);
        self
    }

    /// Register a provider with the application's dependency injection container.
    ///
    /// This method can be used to add providers directly to the container, avoiding the need
    /// to create a full module when only a provider is needed.
    pub fn with_provider<T>(self, provider: T) -> Self
    where
        T: Provider + 'static,
    {
        self.container.provider_registry().register(provider);
        self
    }

    fn build_router(&mut self) -> Router {
        let mut router = Router::new().with_state(self.state.clone());

        #[cfg(feature = "socketio")]
        let layer = {
            use socketioxide::SocketIo;
            use socketioxide::extract::SocketRef;
            use std::any::TypeId;
            use std::sync::Arc;

            let (layer, io) = socketioxide::SocketIo::new_layer();

            io.ns("/test", async |socket: SocketRef| {
                println!("New connection established");

                socket.on("ping", async |socket: SocketRef| {
                    println!("Received 'ping' event");
                    socket.emit("pong", &String::from("pong response")).ok();
                });
            });

            self.state
                .insert_dependency(TypeId::of::<SocketIo>(), Arc::new(io));

            layer
        };

        router = self.apply_gateways(router);

        router = self.apply_sword_layers(router);
        router = self.apply_layers(router);

        #[cfg(feature = "socketio")]
        {
            router = router.layer(layer);
        }

        let app_config = self.config.get::<ApplicationConfig>()
            .expect("Failed to get ApplicationConfig. Ensure it is present in the config file.");

        if let Some(prefix) = app_config.global_prefix {
            router = Router::new().nest(&prefix, router);
        }

        router
    }

    /// Build the `Application` instance with the configured options.
    ///
    /// This method ends the builder pattern and constructs the final `Application`
    /// instance ready to run. The router is built by:
    /// 1. Applying gateways (REST, WebSocket, etc.) to register routes
    /// 2. Applying custom layers added via `with_layer()`
    /// 3. Applying built-in Sword middleware layers (CORS, compression, etc.)
    /// 4. Nesting under a global prefix if configured
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

        self.container.clear();

        Application::new(router, self.config)
    }

    fn apply_layers(&self, router: Router) -> Router {
        self.layer_stack.apply(router)
    }

    // Merge all the "controllers" routers into the main router
    // In fact, controllers are just functions that return a Router
    fn apply_gateways(&self, mut router: Router) -> Router {
        for adapter_kind in self.adapter_registry.adapters.read().iter() {
            match adapter_kind {
                AdapterKind::Rest(builder) => {
                    let gw_router = builder(self.state.clone());
                    router = router.merge(gw_router);
                }
                AdapterKind::WebSocket(setup_fn) => {
                    setup_fn(&self.state);
                }
                AdapterKind::Grpc => {}
            }
        }

        router
    }

    fn apply_sword_layers(&self, router: Router) -> Router {
        let mut router = router;

        let middlewares_config = self
            .config
            .get::<MiddlewaresConfig>()
            .expect("Failed to get MiddlewaresConfig. Ensure it is present in the config file.");

        if middlewares_config.request_timeout.enabled {
            let (timeout_service, response_mapper) =
                RequestTimeoutLayer::new(&middlewares_config.request_timeout);

            router = router.layer(timeout_service);
            router = router.layer(response_mapper);
        }

        if middlewares_config.cors.enabled {
            router = router.layer(CorsLayer::new(&middlewares_config.cors));
        };

        if middlewares_config.compression.enabled {
            router =
                router.layer(CompressionLayer::new(&middlewares_config.compression));
        }

        if middlewares_config.serve_dir.enabled {
            let serve_dir = ServeDirLayer::new(&middlewares_config.serve_dir);

            router = router
                .nest_service(&middlewares_config.serve_dir.router_path, serve_dir);
        }

        router = router.layer(RequestIdLayer::new());
        router = router.layer(CookieManagerLayer::new());

        router
    }
}

impl Default for ApplicationBuilder {
    fn default() -> Self {
        Self::new()
    }
}
