mod provider;

pub use provider::Database;
use serde::Deserialize;
use sword::prelude::*;

#[derive(Clone, Deserialize)]
#[config(key = "db-config")]
pub struct DatabaseConfig {
    pub uri: String,
    pub migrations_path: String,
}

pub struct DatabaseModule;

impl Module for DatabaseModule {
    async fn register_providers(
        config: &Config,
        _: &State,
        container: &mut DependencyContainer,
    ) {
        let db_config = config.get::<DatabaseConfig>().unwrap();
        let database = Database::new(db_config).await;

        container.register_provider(database);
    }
}
