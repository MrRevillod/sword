use std::sync::Arc;

use crate::TaskRepository;
use serde_json::Value;
use sword::core::injectable;

#[injectable]
pub struct TasksService {
    repository: Arc<TaskRepository>,
}

impl TasksService {
    pub async fn create(&self, task: Value) {
        self.repository.create(task).await;
    }

    pub async fn find_all(&self) -> Vec<Value> {
        self.repository.find_all().await.unwrap_or_default()
    }
}
