pub mod chat;
pub mod database;

use sword::prelude::*;

use crate::{chat::ChatModule, database::Database};
use sword_layers::cors::{CorsConfig, CorsLayer};

#[sword::main]
async fn main() {
    let database = Database::new();
    let config = Config::builder()
        .add_file("Config.toml")
        .build()
        .expect("Failed to load configuration");

    let cors_config = config.expect::<CorsConfig>();

    let app = Application::from_config(config)
        .with_module::<ChatModule>()
        .with_provider(database)
        .with_layer(CorsLayer::new(&cors_config))
        .build();

    app.run().await;
}
