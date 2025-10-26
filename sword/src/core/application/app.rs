use axum::routing::Router;
use axum_responses::http::HttpResponse;
#[cfg(feature = "websocket")]
use socketioxide::SocketIo;
use tokio::net::TcpListener as Listener;

use crate::core::{
    ApplicationBuilder, ApplicationConfig, ApplicationError, Config, ConfigError,
    ConfigItem,
};

/// The main application struct that holds the router and configuration.
///
/// `Application` is the core component of the Sword framework that manages
/// the web server, routing, and application configuration. It provides a
/// builder pattern for configuration and methods to run the application.
pub struct Application {
    pub(crate) router: Router,
    pub(crate) config: Config,
    pub(crate) state: crate::core::State,
    #[cfg(feature = "websocket")]
    pub(crate) socket_setups:
        Vec<(&'static str, crate::web::websocket::SocketSetupFn)>,
}

impl Application {
    pub fn new(router: Router, config: Config, state: crate::core::State) -> Self {
        Self {
            router,
            config,
            state,
            #[cfg(feature = "websocket")]
            socket_setups: Vec::new(),
        }
    }

    #[cfg(feature = "websocket")]
    pub(crate) fn with_socket_setup(
        mut self,
        path: &'static str,
        setup_fn: crate::web::websocket::SocketSetupFn,
    ) -> Self {
        self.socket_setups.push((path, setup_fn));
        self
    }

    /// Creates a new application builder for configuring the application.
    ///
    /// This is the starting point for creating a new Sword application.
    /// The builder pattern allows you to configure various aspects of the
    /// application before building the final `Application` instance.
    ///
    /// This function will panic if:
    /// - The configuration file `config/config.toml` cannot be found
    /// - The configuration file contains invalid TOML syntax
    /// - Environment variable interpolation fails
    /// - The configuration cannot be loaded for any other reason
    ///
    /// ### Example
    ///
    /// ```rust,ignore
    /// use sword::prelude::*;
    ///
    /// let app = Application::builder()
    ///     .with_controller::<MyController>()
    ///     .build();
    /// ```
    pub fn builder() -> ApplicationBuilder {
        ApplicationBuilder::new()
    }

    /// Runs the application server.
    ///
    /// This method starts the web server and begins listening for incoming
    /// HTTP requests. It will bind to the host and port specified in the
    /// application configuration and run until the process is terminated.
    ///
    /// If graceful shutdown is enabled in the configuration, it will handle
    /// termination signals and allow ongoing requests to complete before shutting down.
    ///
    /// ### Errors
    ///
    /// This function will return an error if:
    /// - The server fails to bind to the specified address and port
    /// - There are network-related issues during server operation
    /// - The configuration cannot be retrieved from the application state
    ///
    /// ### Example
    ///
    /// ```rust,ignore
    /// use sword::prelude::*;
    ///
    /// #[sword::main]
    /// async fn main() {
    ///     let app = Application::builder()
    ///         .with_controller::<MyController>()
    ///         .build();
    ///     
    ///     app.run().await;
    /// }
    /// ```
    pub async fn run(&self) {
        if self
            .config
            .get::<ApplicationConfig>()
            .expect("Failed to get application config")
            .graceful_shutdown
        {
            self.run_with_graceful_shutdown(Self::graceful_signal())
                .await;
        }

        let listener = self.pre_run().await;

        let router = self.router.clone().fallback(async || {
            HttpResponse::NotFound().message("The requested resource was not found")
        });

        #[cfg(feature = "websocket")]
        {
            let (layer, io) = SocketIo::new_layer();

            // Register all socket setups
            for (_path, setup_fn) in &self.socket_setups {
                // Call the setup function which handles namespace registration 💀
                // This allows handlers to be registered properly for each namespace
                setup_fn(&io, self.state.clone());
            }

            let router = router.layer(layer);

            axum::serve(listener, router)
                .await
                .map_err(|e| ApplicationError::ServerError { source: e })
                .expect("Internal server error");
        }

        #[cfg(not(feature = "websocket"))]
        {
            axum::serve(listener, router)
                .await
                .map_err(|e| ApplicationError::ServerError { source: e })
                .expect("Internal server error");
        }
    }

    /// Runs the application server with graceful shutdown support.
    /// Is similar to `run` but accepts a shutdown signal.
    ///
    /// See [Axum's docs](https://docs.rs/axum/latest/axum/serve/struct.WithGracefulShutdown.html)
    /// to learn more about graceful shutdown.
    ///
    /// To use this method, disable the "graceful shutdown" option on config.toml.
    /// If this option is setted as true the application it will use the default axum's provided
    /// Graceful shutdown signal.
    ///
    /// ### Example
    ///
    /// ```rust,ignore
    /// use sword::prelude::*;
    /// use tokio::signal;
    ///
    /// #[controller("/admin")]
    /// struct AdminController {}
    ///
    /// #[routes]
    /// impl AdminController {
    ///     #[get("/")]
    ///     async fn get_admin_data(&self) -> HttpResponse {
    ///         HttpResponse::Ok()
    ///     }
    /// }
    ///
    /// #[sword::main]
    /// async fn main() {
    ///     let app = Application::builder()
    ///         .with_controller::<AppController>()
    ///         .with_controller::<AdminController>()
    ///         .build();
    ///
    ///     app.run_with_graceful_shutdown(shutdown_signal()).await;
    /// }
    ///
    /// async fn shutdown_signal() {
    ///     let ctrl_c = async {
    ///         signal::ctrl_c()
    ///             .await
    ///             .expect("failed to install Ctrl+C handler");
    ///     };
    ///
    ///     #[cfg(unix)]
    ///     let terminate = async {
    ///         signal::unix::signal(signal::unix::SignalKind::terminate())
    ///             .expect("failed to install signal handler")
    ///             .recv()
    ///             .await;
    ///     };
    ///
    ///     #[cfg(not(unix))]
    ///     let terminate = std::future::pending::<()>();
    ///
    ///     tokio::select! {
    ///         _ = ctrl_c => {},
    ///         _ = terminate => {},
    ///     }
    /// }
    pub async fn run_with_graceful_shutdown<F>(&self, signal: F)
    where
        F: Future<Output = ()> + Send + 'static,
    {
        let listener = self.pre_run().await;

        let router = self.router.clone().fallback(async || {
            HttpResponse::NotFound().message("The requested resource was not found")
        });

        axum::serve(listener, router)
            .with_graceful_shutdown(signal)
            .await
            .map_err(|e| ApplicationError::ServerError { source: e })
            .expect("Internal server error");
    }

    /// Returns a clone of the internal Axum router.
    ///
    /// This method provides access to the underlying Axum router for advanced
    /// use cases where direct router manipulation is needed. Most applications
    /// should not need to use this method directly.
    ///
    /// ### Returns
    ///
    /// A cloned `Router` instance that can be used for testing or integration
    /// with other Axum-based systems.
    ///
    /// ### Example
    ///
    /// ```rust,ignore
    /// use sword::prelude::*;
    ///
    /// let app = Application::builder()
    ///     .with_controller::<MyController>()
    ///     .build();
    ///
    /// let router = app.router();
    /// // Use router for testing or other purposes
    /// ```
    pub fn router(&self) -> Router {
        self.router.clone()
    }

    pub fn config<T>(&self) -> Result<T, ConfigError>
    where
        T: ConfigItem,
    {
        self.config.get::<T>()
    }

    async fn pre_run(&self) -> Listener {
        let config = self
            .config
            .get::<ApplicationConfig>()
            .expect("Failed to get application config");

        let addr = format!("{}:{}", config.host, config.port);

        let listener = Listener::bind(&addr)
            .await
            .map_err(|e| ApplicationError::BindFailed {
                address: addr.to_string(),
                source: e,
            })
            .expect("Failed to bind to address");

        config.display();

        listener
    }

    async fn graceful_signal() {
        let ctrl_c = async {
            tokio::signal::ctrl_c()
                .await
                .expect("failed to install Ctrl+C handler");
        };

        #[cfg(unix)]
        let terminate = async {
            tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
                .expect("failed to install signal handler")
                .recv()
                .await;
        };

        #[cfg(not(unix))]
        let terminate = std::future::pending::<()>();

        tokio::select! {
            _ = ctrl_c => {
                println!(" Shutdown signal received, starting graceful shutdown...");
            },
            _ = terminate => {
                println!(" Shutdown signal received, starting graceful shutdown...");
            },
        }
    }
}
