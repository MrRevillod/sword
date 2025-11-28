pub mod database;
pub mod errors;
pub mod hasher;

pub use database::*;
pub use hasher::*;

use sword::prelude::*;

pub struct SharedModule;

impl Module for SharedModule {
    type Controller = NonControllerModule;

    async fn register_providers(
        config: &Config,
        container: &mut DependencyContainer,
    ) {
        let db_config = config.get::<DatabaseConfig>().unwrap();
        let hasher_config = config.get::<HasherConfig>().unwrap();

        container.register_provider(
            Database::new(db_config)
                .await
                .expect("Failed to create Database provider"),
        );

        container.register_provider(Hasher::new(&hasher_config));
    }
}
