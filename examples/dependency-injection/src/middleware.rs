use std::sync::Arc;
use sword::prelude::*;

use crate::TaskRepository;

#[middleware]
pub struct MyMiddleware {
    tasks_repository: Arc<TaskRepository>,
}

impl MyMiddleware {
    #[on_request]
    async fn handle(&self, req: Request, next: Next) -> MiddlewareResult {
        let tasks = self.tasks_repository.find_all().await;

        println!("Current tasks:");

        match tasks {
            Some(tasks) => println!("{tasks:?}"),
            None => println!("There's no tasks"),
        }

        next!(req, next)
    }
}
