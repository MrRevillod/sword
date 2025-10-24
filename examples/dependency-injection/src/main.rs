mod database;
mod middleware;
mod repository;

use std::sync::Arc;

use dotenv::dotenv;
pub use middleware::MyMiddleware;
pub use repository::TaskRepository;

use sword::{core::DependencyContainer, prelude::*};

use crate::{
    database::{Database, DatabaseConfig},
    repository::Task,
};

#[controller("/tasks")]
struct TasksController {
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

#[sword::main]
async fn main() {
    dotenv().ok();

    let app = Application::builder();
    let db_config = app.config::<DatabaseConfig>().unwrap();

    let db = Database::new(db_config).await;

    let container = DependencyContainer::builder()
        .register_provider(db)
        .register_component::<TaskRepository>()
        .build();

    let app = app
        .with_dependency_container(container)
        .with_controller::<TasksController>()
        .build();

    app.run().await;
}
