use std::{convert::Infallible, time::Duration};

use axum::{
    extract::Request as AxumRequest,
    middleware::from_fn_with_state as mw_with_state,
    response::IntoResponse,
    routing::{Route, Router},
};

use tower::{Layer, Service};
use tower_http::{limit::RequestBodyLimitLayer, timeout::TimeoutLayer};

#[cfg(feature = "cookies")]
use tower_cookies::CookieManagerLayer;

use crate::{
    core::*,
    web::{ContentTypeCheck, Controller, MiddlewareRegistrar, ResponsePrettifier},
};

/// Builder for constructing a Sword application with various configuration options.
///
/// `ApplicationBuilder` provides a fluent interface for configuring a Sword application
/// before building the final `Application` instance. It allows you to register
/// controllers, add middleware layers, configure shared state, and set up dependency injection.
///
/// ### Example
///
/// ```rust,ignore
/// use sword::prelude::*;
///
/// #[controller]
/// struct HomeController;
///
/// let app = Application::builder()
///     .with_controller::<HomeController>()
///     .with_layer(tower_http::cors::CorsLayer::permissive())
///     .build();
/// ```
#[derive(Clone)]
pub struct ApplicationBuilder {
    /// The internal Axum router that handles HTTP requests.
    router: Router,

    /// Shared application state for dependency injection and data sharing.
    state: State,

    /// Application configuration loaded from TOML files.
    config: Config,

    /// Optional URL prefix for all routes in the application.
    prefix: Option<String>,

    /// Flag to track if middlewares have been registered
    middlewares_registered: bool,

    /// Socket.IO setup configurations
    #[cfg(feature = "websocket")]
    socket_setups: Vec<(&'static str, crate::web::websocket::SocketSetupFn)>,
}

impl ApplicationBuilder {
    /// Creates a new application builder with default configuration.
    ///
    /// This method initializes a new builder with:
    /// - Empty router
    /// - Fresh state container
    /// - Configuration loaded from `config/config.toml`
    ///
    /// ### Returns
    ///
    /// Returns `Ok(ApplicationBuilder)` if initialization succeeds, or
    /// `Err(ApplicationError)` if configuration loading fails.
    ///
    /// ### Errors
    ///
    /// This function will return an error if:
    /// - The configuration file cannot be found or read
    /// - The TOML syntax is invalid
    /// - Environment variable interpolation fails
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
            prefix: None,
            middlewares_registered: false,
            #[cfg(feature = "websocket")]
            socket_setups: Vec::new(),
        }
    }

    /// Registers a controller in the application.
    ///
    /// This method adds a controller and its routes to the application's router.
    /// Controllers must implement the `RouterProvider` trait, which is typically
    /// done using the `#[controller]` and `#[routes]` macros.
    ///
    /// ### Type Parameters
    ///
    /// * `R` - A type implementing `RouterProvider` that defines the controller's routes
    ///
    /// ### Example
    ///
    /// ```rust,ignore
    /// use sword::prelude::*;
    ///
    /// #[controller("/")]
    /// struct HomeController;
    ///
    /// #[routes]
    /// impl HomeController {
    ///     #[get("/")]
    ///     async fn index(&self) -> HttpResult<HttpResponse> {
    ///         Ok(HttpResponse::Ok().message("Welcome to the Home Page"))
    ///     }
    /// }
    ///
    /// let app = Application::builder()
    ///     .with_controller::<HomeController>()
    ///     .build();
    /// ```
    pub fn with_controller<C: Controller>(mut self) -> Self {
        if !self.middlewares_registered {
            for MiddlewareRegistrar { register_fn } in
                inventory::iter::<MiddlewareRegistrar>
            {
                (register_fn)(&self.state).expect("Failed to register middleware");
            }

            self.middlewares_registered = true;
        }

        let controller_router = C::router(self.state.clone());
        let router = self.router.clone().merge(controller_router);

        Self {
            router,
            state: self.state,
            config: self.config,
            prefix: self.prefix,
            middlewares_registered: self.middlewares_registered,
            #[cfg(feature = "websocket")]
            socket_setups: self.socket_setups,
        }
    }

    /// Registers a WebSocket gateway in the application.
    ///
    /// This could be used to add WebSocket support.
    /// Currently is used just with Socket.IO via `socketioxide`.
    ///
    /// ### Type Parameters
    ///
    /// * `W` - A type implementing `WebSocketGateway` that defines the WebSocket handlers
    ///
    /// ### Example
    ///
    /// ```rust,ignore
    /// use sword::prelude::*;
    ///
    /// #[web_socket_gateway]
    /// struct SocketController;
    ///
    /// #[web_socket("/socket")]
    /// impl SocketController {
    ///     #[on_connection]
    ///     async fn on_connect(&self, socket: SocketRef) {
    ///         println!("Connected");
    ///     }
    /// }
    ///
    /// let app = Application::builder()
    ///     .with_socket::<SocketController>()
    ///     .build();
    /// ```
    #[cfg(feature = "websocket")]
    pub fn with_socket<W>(self) -> Self
    where
        W: crate::web::websocket::WebSocketProvider + Clone + Send + Sync + 'static,
        W: crate::core::Build,
    {
        let path = W::path();

        if let Ok(controller) = W::build(&self.state) {
            let _ = self.state.insert(controller);
        }

        let setup_fn = W::get_setup_fn(self.state.clone());

        let mut socket_setups = self.socket_setups;
        socket_setups.push((path, setup_fn));

        Self {
            router: self.router,
            state: self.state,
            config: self.config,
            prefix: self.prefix,
            middlewares_registered: self.middlewares_registered,
            #[cfg(feature = "websocket")]
            socket_setups,
        }
    }

    /// Registers a middleware layer in the application.
    ///
    /// This method allows you to add Tower-based middleware or other layers
    /// that implement the `Layer` trait. Layers are applied to all routes
    /// in the application and can modify requests and responses.
    ///
    /// ### Arguments
    ///
    /// * `layer` - The middleware layer to add to the application
    ///
    /// ### Example
    ///
    /// ```rust,ignore
    /// use sword::prelude::*;
    /// use tower_http::cors::CorsLayer;
    /// use tower_http::trace::TraceLayer;
    ///
    /// let app = Application::builder()
    ///     .with_layer(CorsLayer::permissive())
    ///     .with_layer(TraceLayer::new_for_http())
    ///     .build();
    /// ```
    pub fn with_layer<L>(self, layer: L) -> Self
    where
        L: Layer<Route> + Clone + Send + Sync + 'static,
        L::Service: Service<AxumRequest> + Clone + Send + Sync + 'static,
        <L::Service as Service<AxumRequest>>::Response: IntoResponse + 'static,
        <L::Service as Service<AxumRequest>>::Error: Into<Infallible> + 'static,
        <L::Service as Service<AxumRequest>>::Future: Send + 'static,
    {
        let router = self.router.layer(layer);

        Self {
            router,
            state: self.state,
            config: self.config,
            prefix: self.prefix,
            middlewares_registered: self.middlewares_registered,
            #[cfg(feature = "websocket")]
            socket_setups: self.socket_setups,
        }
    }

    /// Registers the provided dependency container in the application.
    ///
    /// Config types marked with `#[config]` are automatically registered in the state
    /// during application initialization, so they can be injected into dependencies.
    ///
    /// This method adds a dependency container to the application, allowing you to
    /// register providers and services that can be resolved later.
    /// ```
    pub fn with_dependency_container(self, container: DependencyContainer) -> Self {
        container
            .build_all(&self.state)
            .unwrap_or_else(|e| panic!("Failed to build dependencies: {e}"));

        Self {
            router: self.router,
            state: self.state,
            config: self.config,
            prefix: self.prefix,
            middlewares_registered: self.middlewares_registered,
            #[cfg(feature = "websocket")]
            socket_setups: self.socket_setups,
        }
    }

    /// Sets a URL prefix for all routes in the application.
    ///
    /// This method allows you to specify a common prefix that will be
    /// applied to all routes registered in the application.
    pub fn with_prefix<S: Into<String>>(self, prefix: S) -> Self {
        Self {
            router: self.router,
            state: self.state,
            config: self.config,
            prefix: Some(prefix.into()),
            middlewares_registered: self.middlewares_registered,
            #[cfg(feature = "websocket")]
            socket_setups: self.socket_setups,
        }
    }

    pub fn config<T>(&self) -> Result<T, ConfigError>
    where
        T: ConfigItem,
    {
        self.config.get::<T>()
    }

    /// Builds the final application instance.
    ///
    /// This method finalizes the application configuration and creates the
    /// `Application` instance. It applies all configured middleware layers,
    /// sets up request body limits, and prepares the application for running.
    ///
    /// ### Built-in Middleware
    ///
    /// The following middleware is automatically applied:
    /// - Content-Type validation middleware
    /// - Request body size limiting middleware
    /// - Cookie management layer (if `cookies` feature is enabled)
    pub fn build(self) -> Application {
        let mut router = self.router.clone();
        let app_config = self.config.get::<ApplicationConfig>().unwrap();

        router = router
            .layer(mw_with_state(self.state.clone(), ContentTypeCheck::layer))
            .layer(RequestBodyLimitLayer::new(app_config.body_limit.parsed));

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

        if let Some(prefix) = &self.prefix {
            router = Router::new().nest(prefix, router);
        }

        let mut app = Application::new(router, self.config, self.state.clone());

        #[cfg(feature = "websocket")]
        {
            for (path, setup_fn) in self.socket_setups {
                app = app.with_socket_setup(path, setup_fn);
            }
        }

        app
    }
}

impl Default for ApplicationBuilder {
    fn default() -> Self {
        Self::new()
    }
}
