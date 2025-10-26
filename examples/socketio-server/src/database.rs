use std::{path::Path, sync::Arc};

use serde::Deserialize;
use sqlx::{migrate::Migrator, PgPool};
use sword::prelude::*;

#[derive(Clone, Deserialize)]
#[config(key = "db-config")]
pub struct DatabaseConfig {
    uri: String,
    migrations_path: String,
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
