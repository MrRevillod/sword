mod config;
mod router;

use crate::{ApplicationConfig, controllers::ControllerRegistry};
use axum::Router;
use sword_core::{
    Config, State,
    layers::{DisplayConfig, LayerStack, NotFoundLayer},
    sword_error,
};
use tokio::net::TcpListener;

pub use config::WebApplicationConfig;
pub use router::WebRouter;

pub struct WebApplication {
    state: State,
    app_config: ApplicationConfig,
    web_config: WebApplicationConfig,
    router: Option<Router<State>>,
}

impl WebApplication {
    pub fn new(
        state: State,
        config: Config,
        layer_stack: LayerStack<State>,
        controller_registry: &ControllerRegistry,
    ) -> Self {
        let app_config = config.get_or_default::<ApplicationConfig>();
        let web_config = config.get_or_default::<WebApplicationConfig>();

        let http_router = WebRouter::new(state.clone(), config.clone());
        let mut router = http_router.build(layer_stack, controller_registry);

        if let Some(prefix) = &web_config.web_router_prefix {
            router = Router::new().nest(prefix, router);
        }

        router = router.layer(NotFoundLayer::new());

        Self {
            state,
            web_config,
            app_config,
            router: Some(router),
        }
    }

    pub async fn start(&self) {
        let router = self
            .router
            .as_ref()
            .unwrap_or_else(|| {
                sword_error! {
                    title: "Axum HTTP router is not initialized",
                    reason: "Router is missing from WebApplication state",
                    context: {
                        "source" => "WebApplication::start",
                    },
                    hints: ["This indicates an internal startup bug, report it with a reproduction"],
                }
            })
            .clone();

        let app = router.with_state(self.state.clone());

        self.display_config();

        let listener = TcpListener::bind(&format!(
            "{}:{}",
            self.web_config.host, self.web_config.port
        ))
        .await
        .unwrap_or_else(|err| {
            sword_error! {
                title: "Failed to bind HTTP listener",
                reason: err,
                context: {
                    "host" => self.web_config.host.clone(),
                    "port" => self.web_config.port.to_string(),
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
                            "host" => self.web_config.host.clone(),
                            "port" => self.web_config.port.to_string(),
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
                    "host" => self.web_config.host.clone(),
                    "port" => self.web_config.port.to_string(),
                },
            }
        });
    }

    pub fn router(&self) -> axum::Router {
        let router = self
            .router
            .as_ref()
            .unwrap_or_else(|| {
                sword_error! {
                    title: "HTTP router is not initialized",
                    reason: "Router is missing from WebApplication state",
                    context: {
                        "source" => "WebApplication::router",
                    },
                    hints: ["This indicates an internal startup bug, report it with a reproduction"],
                }
            })
            .clone();

        router.with_state(self.state.clone())
    }

    fn display_config(&self) {
        println!("\n▪──────────────── ⚔ S W O R D ⚔ ──────────────▪");

        if let Ok(app_config) =
            self.state.get::<crate::application::ApplicationConfig>()
        {
            app_config.display();
        }

        self.web_config.display();

        #[cfg(feature = "socketio")]
        if let Ok(socketio_config) =
            self.state
                .get::<crate::controllers::socketio::SocketIoServerConfig>()
        {
            socketio_config.display();
        }

        println!("\n▪──────────────── ⚔ ───────── ⚔ ──────────────▪\n");
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
                println!(" Shutdown signal received, starting graceful shutdown...");
            },
            _ = terminate => {
                println!(" Shutdown signal received, starting graceful shutdown...");
            },
        }
    }
}
