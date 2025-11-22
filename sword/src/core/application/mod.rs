mod builder;
mod config;

pub use builder::ApplicationBuilder;
pub use config::ApplicationConfig;

use axum::routing::Router;
use axum_responses::http::HttpResponse;
use tokio::net::TcpListener;

use crate::core::{Config, LimitsMiddlewareConfig};

/// The main application struct that holds the router and configuration.
///
/// `Application` is the core component of the Sword framework that manages
/// the web server, routing, and application configuration. It provides a
/// builder pattern for configuration and methods to run the application.
pub struct Application {
    router: Router,
    pub config: Config,
}

impl Application {
    pub fn new(router: Router, config: Config) -> Self {
        Self { router, config }
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

        let listener = self.build_listener().await;

        let router = self.router.clone().fallback(async || {
            HttpResponse::NotFound().message("The requested resource was not found")
        });

        axum::serve(listener, router)
            .await
            .expect("Internal 'axum::serve' error");
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
    pub async fn run_with_graceful_shutdown<F>(&self, signal: F)
    where
        F: Future<Output = ()> + Send + 'static,
    {
        let listener = self.build_listener().await;

        let router = self.router.clone().fallback(async || {
            HttpResponse::NotFound().message("The requested resource was not found")
        });

        axum::serve(listener, router)
            .with_graceful_shutdown(signal)
            .await
            .expect("Internal 'axum::serve' error");
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
    pub fn router(&self) -> Router {
        self.router.clone()
    }

    async fn build_listener(&self) -> TcpListener {
        let app_config = self
            .config
            .get::<ApplicationConfig>()
            .expect("Failed to get application config");

        let limits_config = self
            .config
            .get::<LimitsMiddlewareConfig>()
            .expect("Failed to get limits middleware config");

        app_config.display();
        limits_config.display();

        let ApplicationConfig { host, port, .. } = app_config;

        TcpListener::bind(&format!("{host}:{port}"))
            .await
            .expect("Failed to bind to address")
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
