mod config;
mod router;

use crate::{application::ApplicationConfig, controllers::ControllerRegistry};

use axum::Router;
use sword_core::{Config, State, sword_error};
use sword_layers::layer_stack::LayerStack;
use tokio::net::TcpListener;

pub use config::WebApplicationConfig;
pub(crate) use router::WebRouter;

pub struct WebApplication {
    state: State,
    router: Router<State>,
    app_config: ApplicationConfig,
}

impl WebApplication {
    pub fn new(
        state: State,
        config: &Config,
        layers: LayerStack<State>,
        controllers: &ControllerRegistry,
    ) -> Self {
        let app_config = config.get_or_default::<ApplicationConfig>();

        let router = WebRouter {
            state: state.clone(),
            config,
            layer_stack: layers,
            controller_registry: controllers,
        };

        Self {
            state,
            app_config,
            router: router.build(),
        }
    }

    pub async fn start(&self) {
        let bind = format!("{}:{}", self.app_config.web.host, self.app_config.web.port);

        tracing::info!(
            target: "sword.startup.web",
            bind,
            web_router_prefix = self
                .app_config
                .web
                .web_router_prefix
                .as_deref()
                .unwrap_or("none"),
            "Starting application listener"
        );

        let app = self.router.clone().with_state(self.state.clone());

        let listener = TcpListener::bind(&format!(
            "{}:{}",
            self.app_config.web.host, self.app_config.web.port
        ))
        .await
        .unwrap_or_else(|err| {
            sword_error! {
                title: "Failed to bind HTTP listener",
                reason: err,
                context: {
                    "host" => self.app_config.web.host.clone(),
                    "port" => self.app_config.web.port.to_string(),
                },
                hints: ["Ensure the host/port is available and not already in use"],
            }
        });

        if self.app_config.graceful_shutdown {
            axum::serve(listener, app)
                .with_graceful_shutdown(Self::graceful_signal())
                .await
                .unwrap_or_else(|err| {
                    sword_error! {
                        title: "HTTP server stopped with an internal error",
                        reason: err,
                        context: {
                            "mode" => "graceful_shutdown",
                            "host" => self.app_config.web.host.clone(),
                            "port" => self.app_config.web.port.to_string(),
                        },
                    }
                });

            return;
        }

        axum::serve(listener, app).await.unwrap_or_else(|err| {
            sword_error! {
                title: "HTTP server stopped with an internal error",
                reason: err,
                context: {
                    "mode" => "normal",
                    "host" => self.app_config.web.host.clone(),
                    "port" => self.app_config.web.port.to_string(),
                },
            }
        });
    }

    pub fn router(&self) -> axum::Router {
        self.router.clone().with_state(self.state.clone())
    }

    async fn graceful_signal() {
        let ctrl_c = async {
            tokio::signal::ctrl_c().await.unwrap_or_else(|err| {
                sword_error! {
                    title: "Failed to install Ctrl+C handler",
                    reason: err,
                    context: {
                        "source" => "WebApplication::graceful_signal",
                    },
                }
            });
        };

        #[cfg(unix)]
        let terminate = async {
            tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
                .unwrap_or_else(|err| {
                    sword_error! {
                        title: "Failed to install SIGTERM handler",
                        reason: err,
                        context: {
                            "signal" => "SIGTERM",
                            "source" => "WebApplication::graceful_signal",
                        },
                    }
                })
                .recv()
                .await;
        };

        #[cfg(not(unix))]
        let terminate = std::future::pending::<()>();

        tokio::select! {
            _ = ctrl_c => {
                tracing::info!(
                    target: "sword.startup.signal",
                    signal = "SIGINT",
                    "Shutdown signal received, starting graceful shutdown"
                );
            },
            _ = terminate => {
                tracing::info!(
                    target: "sword.startup.signal",
                    signal = "SIGTERM",
                    "Shutdown signal received, starting graceful shutdown"
                );
            },
        }
    }
}
