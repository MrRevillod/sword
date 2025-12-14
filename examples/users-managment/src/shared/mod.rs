pub mod database;
pub mod errors;
pub mod hasher;

pub use database::*;
pub use hasher::*;

use sword::prelude::*;

pub struct SharedModule;

impl Module for SharedModule {
    async fn register_providers(config: &Config, container: &DependencyContainer) {
        let db_config = config.get_or_panic::<DatabaseConfig>();
        let hasher_config = config.get_or_default::<HasherConfig>();

        container.register_provider(
            Database::new(db_config)
                .await
                .expect("Failed to create Database provider"),
        );

        container.register_provider(Hasher::new(&hasher_config));
    }
}
