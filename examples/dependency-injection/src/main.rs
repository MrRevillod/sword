mod database;
mod middleware;
mod repository;
mod service;

use std::sync::Arc;

// pub use middleware::MyMiddleware;
pub use repository::TaskRepository;

use serde_json::json;
use sword::{core::DependencyContainer, prelude::*};

use crate::{
    database::{Database, DatabaseConfig},
    service::TasksService,
};

#[controller("/tasks", version = "v1")]
struct TasksController {
    tasks: Arc<TasksService>,
}

#[routes]
impl TasksController {
    #[get("/")]
    // #[middleware(MyMiddleware)]
    async fn get_tasks(&self) -> HttpResponse {
        let data = self.tasks.find_all().await;

        HttpResponse::Ok().data(data)
    }

    #[post("/")]
    async fn create_task(&self) -> HttpResponse {
        let total_task = self.tasks.find_all().await.len();

        let task = json!({
            "id": total_task + 1,
            "title": format!("Task {}", total_task + 1),
        });

        self.tasks.create(task.clone()).await;

        HttpResponse::Created().message("Task created").data(task)
    }
}

#[sword::main]
async fn main() {
    let app = Application::builder();
    let db_config = app.config::<DatabaseConfig>().unwrap();

    let db = Database::new(db_config).await;

    let container = DependencyContainer::builder()
        .register_provider(db)
        .register_component::<TaskRepository>()
        .register_component::<TasksService>()
        .build();

    let app = app
        .with_dependency_container(container)
        .with_controller::<TasksController>()
        .build();

    app.run().await;
}
