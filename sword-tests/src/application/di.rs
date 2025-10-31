use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use axum_test::TestServer;
use serde_json::{Value, json};
use sword::prelude::*;

pub type Store = Arc<RwLock<HashMap<&'static str, Vec<Value>>>>;

#[injectable(provider)]
pub struct Database {
    db: Store,
}

impl Database {
    pub fn new() -> Self {
        let db = Arc::new(RwLock::new(HashMap::new()));

        db.write().unwrap().insert("tasks", Vec::new());

        Self { db }
    }

    pub async fn insert(&self, table: &'static str, record: Value) {
        let mut db = self.db.write().unwrap();

        if let Some(table_data) = db.get_mut(table) {
            table_data.push(record);
        }
    }

    pub async fn get_all(&self, table: &'static str) -> Option<Vec<Value>> {
        let db = self.db.read().unwrap();

        db.get(table).cloned()
    }
}

#[injectable]
pub struct TasksService {
    repository: TaskRepository,
}

impl TasksService {
    pub async fn create(&self, task: Value) {
        self.repository.create(task).await;
    }

    pub async fn find_all(&self) -> Vec<Value> {
        self.repository.find_all().await.unwrap_or_default()
    }
}

#[injectable]
pub struct TaskRepository {
    db: Database,
}

impl TaskRepository {
    pub async fn create(&self, task: Value) {
        self.db.insert("tasks", task).await;
    }

    pub async fn find_all(&self) -> Option<Vec<Value>> {
        self.db.get_all("tasks").await
    }
}

#[controller("/tasks", version = "v1")]
pub struct TasksController {
    tasks: TasksService,
}

#[routes]
impl TasksController {
    #[get("/")]
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

pub struct TasksModule;

impl Module for TasksModule {
    type Controller = TasksController;

    fn register_components(c: &mut DependencyContainer) {
        c.register_component::<TaskRepository>();
        c.register_component::<TasksService>();
    }
}

#[tokio::test]
async fn test_get_tasks_empty() {
    let mut app = Application::builder();

    let db = Database::new();

    app = app.with_provider(db).with_module::<TasksModule>();

    let server = TestServer::new(app.build().router()).unwrap();

    let response = server.get("/v1/tasks").await;

    assert_eq!(response.status_code(), StatusCode::OK);

    let body: ResponseBody = response.json();

    assert!(body.success);
    assert_eq!(body.code, 200);
    assert_eq!(body.data, Some(json!([])));
}

#[tokio::test]
async fn test_create_task() {
    let mut app = Application::builder();

    let db = Database::new();

    app = app.with_provider(db).with_module::<TasksModule>();

    let server = TestServer::new(app.build().router()).unwrap();

    let response = server.post("/v1/tasks").await;

    assert_eq!(response.status_code(), 201);

    let body: ResponseBody = response.json();

    assert!(body.success);
    assert_eq!(body.code, 201);
    assert_eq!(body.message.as_ref(), "Task created");

    let task = body.data.unwrap();
    assert_eq!(task["id"], 1);
    assert_eq!(task["title"], "Task 1");
}
