use std::sync::Arc;
use sword::prelude::*;

use crate::tasks::{middleware::MyMiddleware, Task, TaskRepository};

#[controller("/tasks")]
pub struct TasksController {
    tasks: Arc<TaskRepository>,
}

#[routes]
impl TasksController {
    #[get("/")]
    #[uses(MyMiddleware)]
    async fn get_tasks(&self) -> HttpResponse {
        let data = self.tasks.find_all().await;

        HttpResponse::Ok().data(data)
    }

    #[post("/")]
    async fn create_task(&self) -> HttpResponse {
        let tasks = self.tasks.find_all().await;
        let total_count = tasks.len() as i32 + 1;

        let task = Task {
            id: total_count,
            title: format!("Task {total_count}"),
        };

        self.tasks.create(task.clone()).await;

        HttpResponse::Created().message("Task created").data(task)
    }
}
