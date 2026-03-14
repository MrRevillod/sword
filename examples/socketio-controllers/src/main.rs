pub mod chat;
pub mod database;

use sword::prelude::*;
use tracing_subscriber::EnvFilter;

use crate::{chat::ChatModule, database::Database};

#[sword::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("info,sword=info")),
        )
        .with_target(false)
        .init();

    let database = Database::new();

    let app = Application::from_config_path("Config.toml")
        .with_module::<ChatModule>()
        .with_provider(database)
        .build();

    app.run().await;
}
