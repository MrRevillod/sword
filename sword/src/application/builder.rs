use crate::adapters::AdapterRegistry;
use crate::application::Application;
use crate::interceptor::InterceptorRegistrar;
use crate::module::Module;
use crate::runtimes::http::HttpRuntime;

use axum::{
    extract::Request as AxumRequest, response::IntoResponse, routing::Route,
};
use std::convert::Infallible;
use sword_core::layers::LayerStack;
use sword_core::{Config, ConfigRegistrar, DependencyContainer, Provider, State};
use tower::{Layer, Service};

pub struct ApplicationBuilder {
    state: State,
    container: DependencyContainer,
    adapter_registry: AdapterRegistry,
    layer_stack: LayerStack<State>,
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
        Self::from_config(Config::new().expect("Configuration loading error"))
    }

    /// Builder for constructing a Sword application with a provided configuration.
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
    /// let app = Application::from_config(Config::new().expect("Configuration loading error"))
    ///     .with_module::<UsersModule>()
    ///     .with_module::<ProductsModule>()
    ///     .with_layer(custom_middleware)
    ///     .build();
    ///
    /// app.run().await;
    /// ```
    pub fn from_config(config: Config) -> Self {
        let state = State::new();
        state.insert(config.clone());

        for ConfigRegistrar { register } in inventory::iter::<ConfigRegistrar> {
            register(&state, &config)
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
        futures_lite::future::block_on(M::register_providers(
            &self.config,
            self.container.provider_registry(),
        ));

        M::register_components(self.container.component_registry());
        M::register_adapters(&self.adapter_registry);

        self
    }

    /// Adds a `tower::Layer` to the application builder.
    ///
    /// This method is equivalent to Axum's `Router::layer` method, allowing you to
    /// apply middleware layers to the application's router.
    ///
    /// Custom layers are applied **after** all built-in Sword middlewares,
    /// making them the outermost layer in the middleware stack.
    /// This means custom layers execute first on incoming requests and last on outgoing responses.
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

    /// Register a provider directly with the application builder.
    ///
    /// This method can be used to add providers directly to the application, avoiding the need
    /// to create a full module when only a provider is needed.
    pub fn with_provider<T>(self, provider: T) -> Self
    where
        T: Provider + 'static,
    {
        self.container.provider_registry().register(provider);
        self
    }

    /// Build the `Application` instance with the configured options.
    ///
    /// This method ends the builder pattern and constructs the final `Application`
    /// instance ready to run.
    pub fn build(mut self) -> Application {
        self.container
            .build_all(&self.state)
            .unwrap_or_else(|err| {
                eprintln!("\n[!] Failed to build dependency injection container\n");
                eprintln!("    Error: {}\n", err);
                eprintln!("    Check that all required components and providers are registered correctly.\n");
                panic!("DI container build failed");
            });

        for InterceptorRegistrar { register } in
            inventory::iter::<InterceptorRegistrar>
        {
            register(&self.state);
        }

        let layer_stack = std::mem::take(&mut self.layer_stack);

        let http_runtime = HttpRuntime::new(
            self.state.clone(),
            self.config.clone(),
            layer_stack,
            &self.adapter_registry,
        );

        Application::new(http_runtime, self.config)
    }
}

impl Default for ApplicationBuilder {
    fn default() -> Self {
        Self::new()
    }
}
