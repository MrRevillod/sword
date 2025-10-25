use std::sync::Arc;
use sword::prelude::*;

use crate::TaskRepository;

#[middleware]
pub struct MyMiddleware {
    tasks_repository: Arc<TaskRepository>,
}

impl OnRequest for MyMiddleware {
    async fn on_request(&self, req: Request, next: Next) -> MiddlewareResult {
        let tasks = self.tasks_repository.find_all().await;

        println!();
        println!("Current tasks:");

        if tasks.is_empty() {
            println!("There's no tasks");
        }

        for task in tasks {
            println!(" - [{}] {}", task.id, task.title);
        }

        req.run(next).await
    }
}
