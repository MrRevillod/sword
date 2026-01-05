use std::{collections::HashMap, sync::Arc};
use sword::prelude::*;
use tokio::sync::RwLock;

use crate::chat::Message;

#[injectable(provider)]
pub struct Database {
    pool: Arc<RwLock<HashMap<String, Message>>>,
}

impl Database {
    pub fn new() -> Self {
        Database {
            pool: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn get(&self, key: &str) -> Option<Message> {
        let pool = self.pool.read().await;

        pool.get(key).cloned()
    }

    pub async fn get_all(&self) -> Vec<Message> {
        let pool = self.pool.read().await;

        pool.values().cloned().collect()
    }

    pub async fn set(&self, key: &str, value: Message) {
        let mut pool = self.pool.write().await;

        pool.insert(key.to_string(), value);
    }
}

impl Default for Database {
    fn default() -> Self {
        Self::new()
    }
}
