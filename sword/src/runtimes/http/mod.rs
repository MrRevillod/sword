mod config;
mod router;

#[cfg(feature = "adapter-socketio")]
mod socketio_config;

use crate::{ApplicationConfig, adapters::AdapterRegistry};
use axum::Router;
use sword_core::{
    Config, State,
    layers::{LayerStack, NotFoundLayer},
};
use sword_layers::DisplayConfig;
use tokio::net::TcpListener;

pub use config::HttpRuntimeConfig;
pub use router::HttpRouter;

#[cfg(feature = "adapter-socketio")]
pub use socketio_config::{SocketIoParser, SocketIoServerConfig};

pub struct HttpRuntime {
    state: State,
    app_config: ApplicationConfig,
    runtime_config: HttpRuntimeConfig,
    router: Option<Router<State>>,
}

impl HttpRuntime {
    pub fn new(
        state: State,
        config: Config,
        layer_stack: LayerStack<State>,
        adapter_registry: &AdapterRegistry,
    ) -> Self {
        let app_config = config.get_or_default::<ApplicationConfig>();
        let runtime_config = config.get_or_default::<HttpRuntimeConfig>();

        let http_router = HttpRouter::new(state.clone(), config.clone());
        let mut router = http_router.build(layer_stack, adapter_registry);

        if let Some(prefix) = &runtime_config.http_router_prefix {
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
            .expect("Router not initialized")
            .clone();

        let app = router.with_state(self.state.clone());

        self.display_config();

        let listener = TcpListener::bind(&format!(
            "{}:{}",
            self.runtime_config.host, self.runtime_config.port
        ))
        .await
        .expect("Failed to bind to address");

        if self.app_config.graceful_shutdown {
            axum::serve(listener, app)
                .with_graceful_shutdown(Self::graceful_signal())
                .await
                .expect("Internal 'axum::serve' error");

            return;
        }

        axum::serve(listener, app)
            .await
            .expect("Internal 'axum::serve' error");
    }

    pub fn router(&self) -> axum::Router {
        let router = self
            .router
            .as_ref()
            .expect("Router not initialized")
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

        if let Ok(middlewares) =
            self.state.get::<sword_core::layers::MiddlewaresConfig>()
        {
            middlewares.display();
        }

        #[cfg(feature = "adapter-socketio")]
        if let Ok(socketio_config) = self.state.get::<SocketIoServerConfig>() {
            socketio_config.display();
        }

        println!("\n▪──────────────── ⚔ ───────── ⚔ ──────────────▪\n");
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
