use std::sync::Arc;

use crate::database::{Database, DatabaseConfig};
use serde_json::Value;

use sword::core::injectable;

#[injectable]
pub struct TaskRepository {
    db_conf: DatabaseConfig,
    db: Arc<Database>,
}

impl TaskRepository {
    pub async fn create(&self, task: Value) {
        self.db.insert("tasks", task).await;
    }

    pub async fn find_all(&self) -> Option<Vec<Value>> {
        self.db.get_all("tasks").await
    }
}
