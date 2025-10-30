use serde::Deserialize;
use sqlx::{migrate::Migrator, PgPool};
use std::{path::Path, sync::Arc};
use sword::prelude::*;

#[derive(Clone, Deserialize)]
#[config(key = "database")]
pub struct DatabaseConfig {
    pub uri: String,
    pub migrations_path: String,
}

#[injectable(provider)]
pub struct Database {
    pool: Arc<PgPool>,
}

impl Database {
    pub async fn new(db_conf: DatabaseConfig) -> Self {
        let pool = PgPool::connect(&db_conf.uri)
            .await
            .expect("Failed to create Postgres connection pool");

        let migrator = Migrator::new(Path::new(&db_conf.migrations_path))
            .await
            .unwrap();

        migrator
            .run(&pool)
            .await
            .expect("Failed to run database migrations");

        Self {
            pool: Arc::new(pool),
        }
    }

    pub fn get_pool(&self) -> &PgPool {
        &self.pool
    }
}
