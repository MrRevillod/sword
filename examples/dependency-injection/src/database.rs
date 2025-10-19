use std::{collections::HashMap, sync::Arc};

use serde::Deserialize;
use serde_json::Value;
use sword::prelude::*;
use tokio::sync::RwLock;

pub type Store = Arc<RwLock<HashMap<String, Vec<Value>>>>;

#[derive(Clone, Deserialize)]
#[config(key = "db-config")]
pub struct DatabaseConfig {
    collection_name: String,
}

#[injectable(kind = "provider")]
pub struct Database {
    db: Store,
}

impl Database {
    pub async fn new(db_conf: DatabaseConfig) -> Self {
        let db = Arc::new(RwLock::new(HashMap::new()));

        db.write().await.insert(db_conf.collection_name, Vec::new());

        Self { db }
    }

    pub async fn insert(&self, table: &'static str, record: Value) {
        let mut db = self.db.write().await;

        if let Some(table_data) = db.get_mut(table) {
            table_data.push(record);
        }
    }

    pub async fn get_all(&self, table: &'static str) -> Option<Vec<Value>> {
        let db = self.db.read().await;

        db.get(table).cloned()
    }
}
