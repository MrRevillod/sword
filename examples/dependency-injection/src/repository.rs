use crate::database::Database;

use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use std::sync::Arc;
use sword::core::injectable;

#[derive(Clone, Serialize, Deserialize, FromRow)]
pub struct Task {
    pub id: i32,
    pub title: String,
}

#[injectable]
pub struct TaskRepository {
    db: Arc<Database>,
}

impl TaskRepository {
    pub async fn find_all(&self) -> Vec<Task> {
        sqlx::query_as::<_, Task>("SELECT id, title FROM tasks")
            .fetch_all(self.db.get_pool())
            .await
            .expect("Failed to fetch tasks")
    }

    pub async fn create(&self, task: Task) {
        sqlx::query("INSERT INTO tasks (id, title) VALUES ($1, $2)")
            .bind(task.id)
            .bind(task.title)
            .execute(self.db.get_pool())
            .await
            .expect("Failed to insert task");
    }
}
