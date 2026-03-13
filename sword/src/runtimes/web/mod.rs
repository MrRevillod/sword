mod config;
mod router;

#[cfg(feature = "web-adapter-socketio")]
mod socketio_config;

use crate::{ApplicationConfig, adapters::AdapterRegistry};
use axum::Router;
use sword_core::{
    Config, StartupPhase, State,
    layers::{DisplayConfig, LayerStack, NotFoundLayer},
    sword_error,
};
use tokio::net::TcpListener;

pub use config::WebRuntimeConfig;
pub use router::WebRouter;

#[cfg(feature = "web-adapter-socketio")]
pub use socketio_config::{SocketIoParser, SocketIoServerConfig};

pub struct WebRuntime {
    state: State,
    app_config: ApplicationConfig,
    runtime_config: WebRuntimeConfig,
    router: Option<Router<State>>,
}

impl WebRuntime {
    pub fn new(
        state: State,
        config: Config,
        layer_stack: LayerStack<State>,
        adapter_registry: &AdapterRegistry,
    ) -> Self {
        let app_config = config.get_or_default::<ApplicationConfig>();
        let runtime_config = config.get_or_default::<WebRuntimeConfig>();

        let http_router = WebRouter::new(state.clone(), config.clone());
        let mut router = http_router.build(layer_stack, adapter_registry);

        if let Some(prefix) = &runtime_config.web_router_prefix {
            router = Router::new().nest(prefix, router);
        }

        router = router.layer(NotFoundLayer::new());

        Self {
            state,
            runtime_config,
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
                    phase: StartupPhase::Runtime,
                    title: "HTTP router is not initialized",
                    reason: "Router is missing from WebRuntime state",
                    context: {
                        "source" => "WebRuntime::start",
                    },
                    hints: ["This indicates an internal startup bug, report it with a reproduction"],
                }
            })
            .clone();

        let app = router.with_state(self.state.clone());

        self.display_config();

        let listener = TcpListener::bind(&format!(
            "{}:{}",
            self.runtime_config.host, self.runtime_config.port
        ))
        .await
        .unwrap_or_else(|err| {
            sword_error! {
                phase: StartupPhase::Runtime,
                title: "Failed to bind HTTP listener",
                reason: err,
                context: {
                    "host" => self.runtime_config.host.clone(),
                    "port" => self.runtime_config.port.to_string(),
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
                        phase: StartupPhase::Runtime,
                        title: "HTTP server stopped with an internal error",
                        reason: err,
                        context: {
                            "mode" => "graceful_shutdown",
                            "host" => self.runtime_config.host.clone(),
                            "port" => self.runtime_config.port.to_string(),
                        },
                    }
                });

            return;
        }

        axum::serve(listener, app).await.unwrap_or_else(|err| {
            sword_error! {
                phase: StartupPhase::Runtime,
                title: "HTTP server stopped with an internal error",
                reason: err,
                context: {
                    "mode" => "normal",
                    "host" => self.runtime_config.host.clone(),
                    "port" => self.runtime_config.port.to_string(),
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
                    phase: StartupPhase::Runtime,
                    title: "HTTP router is not initialized",
                    reason: "Router is missing from WebRuntime state",
                    context: {
                        "source" => "WebRuntime::router",
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

        self.runtime_config.display();

        #[cfg(feature = "web-adapter-socketio")]
        if let Ok(socketio_config) = self.state.get::<SocketIoServerConfig>() {
            socketio_config.display();
        }

        println!("\n▪──────────────── ⚔ ───────── ⚔ ──────────────▪\n");
    }

    async fn graceful_signal() {
        let ctrl_c = async {
            tokio::signal::ctrl_c().await.unwrap_or_else(|err| {
                sword_error! {
                    phase: StartupPhase::Runtime,
                    title: "Failed to install Ctrl+C handler",
                    reason: err,
                    context: {
                        "source" => "WebRuntime::graceful_signal",
                    },
                }
            });
        };

        #[cfg(unix)]
        let terminate = async {
            tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
                .unwrap_or_else(|err| {
                    sword_error! {
                        phase: StartupPhase::Runtime,
                        title: "Failed to install SIGTERM handler",
                        reason: err,
                        context: {
                            "signal" => "SIGTERM",
                            "source" => "WebRuntime::graceful_signal",
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
