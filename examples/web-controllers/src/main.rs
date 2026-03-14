pub mod shared;
pub mod users;

use dotenv::dotenv;
use sword::prelude::*;
use tracing_subscriber::EnvFilter;

use crate::{shared::SharedModule, users::UsersModule};

#[sword::main]
async fn main() {
    dotenv().ok();

    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("info,sword=info,sqlx=off")),
        )
        .with_target(false)
        .init();

    tracing::info!("Starting Users Management example...");

    let app = Application::builder()
        .with_module::<SharedModule>()
        .with_module::<UsersModule>()
        .build();

    app.run().await;
}
