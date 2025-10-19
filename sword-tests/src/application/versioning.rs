use axum_test::TestServer;
use serde_json::{Value, json};
use sword::prelude::*;

fn test_server() -> TestServer {
    let app = Application::builder()
        .with_controller::<V1UsersController>()
        .with_controller::<V2UsersController>()
        .with_controller::<V1ProductsController>()
        .with_controller::<V2ProductsController>()
        .build();

    TestServer::new(app.router()).unwrap()
}

// ================== V1 Controllers ==================

#[controller("/users", version = "v1")]
pub struct V1UsersController {}

#[routes]
impl V1UsersController {
    #[get("/")]
    async fn list_users(&self) -> HttpResponse {
        HttpResponse::Ok()
            .data(json!({
                "version": "v1",
                "users": [
                    {"id": 1, "name": "Alice"},
                    {"id": 2, "name": "Bob"}
                ]
            }))
            .message("Users retrieved (v1)")
    }

    #[get("/{id}")]
    async fn get_user(&self, req: Request) -> HttpResult {
        let id = req.param::<u32>("id")?;

        Ok(HttpResponse::Ok()
            .data(json!({
                "version": "v1",
                "user": {
                    "id": id,
                    "name": "Alice"
                }
            }))
            .message("User retrieved (v1)"))
    }

    #[post("/")]
    async fn create_user(&self, req: Request) -> HttpResult {
        let body = req.body::<Value>()?;

        Ok(HttpResponse::Created()
            .data(json!({
                "version": "v1",
                "user": body
            }))
            .message("User created (v1)"))
    }
}

#[controller("/products", version = "v1")]
pub struct V1ProductsController {}

#[routes]
impl V1ProductsController {
    #[get("/")]
    async fn list_products(&self) -> HttpResponse {
        HttpResponse::Ok()
            .data(json!({
                "version": "v1",
                "products": [
                    {"id": 1, "name": "Laptop", "price": 999.99},
                    {"id": 2, "name": "Mouse", "price": 29.99}
                ]
            }))
            .message("Products retrieved (v1)")
    }

    #[get("/{id}")]
    async fn get_product(&self, req: Request) -> HttpResult {
        let id = req.param::<u32>("id")?;

        Ok(HttpResponse::Ok()
            .data(json!({
                "version": "v1",
                "product": {
                    "id": id,
                    "name": "Laptop",
                    "price": 999.99
                }
            }))
            .message("Product retrieved (v1)"))
    }
}

// ================== V2 Controllers ==================

#[controller("/users", version = "v2")]
pub struct V2UsersController {}

#[routes]
impl V2UsersController {
    #[get("/")]
    async fn list_users(&self) -> HttpResponse {
        HttpResponse::Ok()
            .data(json!({
                "version": "v2",
                "users": [
                    {"id": 1, "name": "Alice", "email": "alice@example.com", "active": true},
                    {"id": 2, "name": "Bob", "email": "bob@example.com", "active": true}
                ],
                "meta": {
                    "total": 2,
                    "page": 1
                }
            }))
            .message("Users retrieved (v2)")
    }

    #[get("/{id}")]
    async fn get_user(&self, req: Request) -> HttpResult {
        let id = req.param::<u32>("id")?;

        Ok(HttpResponse::Ok()
            .data(json!({
                "version": "v2",
                "user": {
                    "id": id,
                    "name": "Alice",
                    "email": "alice@example.com",
                    "active": true,
                    "created_at": "2025-10-06T00:00:00Z"
                }
            }))
            .message("User retrieved (v2)"))
    }

    #[post("/")]
    async fn create_user(&self, req: Request) -> HttpResult {
        let body = req.body::<Value>()?;

        Ok(HttpResponse::Created()
            .data(json!({
                "version": "v2",
                "user": body,
                "meta": {
                    "created_at": "2025-10-06T00:00:00Z"
                }
            }))
            .message("User created (v2)"))
    }
}

#[controller("/products", version = "v2")]
pub struct V2ProductsController {}

#[routes]
impl V2ProductsController {
    #[get("/")]
    async fn list_products(&self) -> HttpResponse {
        HttpResponse::Ok()
            .data(json!({
                "version": "v2",
                "products": [
                    {
                        "id": 1,
                        "name": "Laptop",
                        "price": 999.99,
                        "currency": "USD",
                        "stock": 50,
                        "category": "Electronics"
                    },
                    {
                        "id": 2,
                        "name": "Mouse",
                        "price": 29.99,
                        "currency": "USD",
                        "stock": 200,
                        "category": "Accessories"
                    }
                ],
                "meta": {
                    "total": 2,
                    "page": 1
                }
            }))
            .message("Products retrieved (v2)")
    }

    #[get("/{id}")]
    async fn get_product(&self, req: Request) -> HttpResult {
        let id = req.param::<u32>("id")?;

        Ok(HttpResponse::Ok()
            .data(json!({
                "version": "v2",
                "product": {
                    "id": id,
                    "name": "Laptop",
                    "price": 999.99,
                    "currency": "USD",
                    "stock": 50,
                    "category": "Electronics",
                    "description": "High-performance laptop"
                }
            }))
            .message("Product retrieved (v2)"))
    }
}

// ================== Tests ==================

#[tokio::test]
async fn test_v1_users_list() {
    let server = test_server();

    let response = server.get("/v1/users").await;

    response.assert_status_ok();

    let json_value = response.json::<Value>();
    assert_eq!(json_value["success"], true);
    assert_eq!(json_value["message"], "Users retrieved (v1)");
    assert_eq!(json_value["data"]["version"], "v1");
    assert_eq!(json_value["data"]["users"][0]["name"], "Alice");
    assert_eq!(json_value["data"]["users"][1]["name"], "Bob");
}

#[tokio::test]
async fn test_v2_users_list() {
    let server = test_server();

    let response = server.get("/v2/users").await;

    response.assert_status_ok();

    let json_value = response.json::<Value>();
    assert_eq!(json_value["success"], true);
    assert_eq!(json_value["message"], "Users retrieved (v2)");
    assert_eq!(json_value["data"]["version"], "v2");
    assert_eq!(json_value["data"]["users"][0]["email"], "alice@example.com");
    assert_eq!(json_value["data"]["meta"]["total"], 2);
}

#[tokio::test]
async fn test_v1_users_get_by_id() {
    let server = test_server();

    let response = server.get("/v1/users/1").await;

    response.assert_status_ok();

    let json_value = response.json::<Value>();
    assert_eq!(json_value["success"], true);
    assert_eq!(json_value["message"], "User retrieved (v1)");
    assert_eq!(json_value["data"]["version"], "v1");
    assert_eq!(json_value["data"]["user"]["id"], 1);
    assert_eq!(json_value["data"]["user"]["name"], "Alice");
}

#[tokio::test]
async fn test_v2_users_get_by_id() {
    let server = test_server();

    let response = server.get("/v2/users/42").await;

    response.assert_status_ok();

    let json_value = response.json::<Value>();
    assert_eq!(json_value["data"]["version"], "v2");
    assert_eq!(json_value["data"]["user"]["id"], 42);
    assert_eq!(json_value["data"]["user"]["email"], "alice@example.com");
    assert!(json_value["data"]["user"]["created_at"].is_string());
}

#[tokio::test]
async fn test_v1_users_create() {
    let server = test_server();

    let response = server
        .post("/v1/users")
        .json(&json!({
            "name": "Charlie"
        }))
        .await;

    assert_eq!(response.status_code(), 201);

    let json_value = response.json::<Value>();
    assert_eq!(json_value["message"], "User created (v1)");
    assert_eq!(json_value["data"]["version"], "v1");
    assert_eq!(json_value["data"]["user"]["name"], "Charlie");
}

#[tokio::test]
async fn test_v2_users_create() {
    let server = test_server();

    let response = server
        .post("/v2/users")
        .json(&json!({
            "name": "Charlie",
            "email": "charlie@example.com"
        }))
        .await;

    assert_eq!(response.status_code(), 201);

    let json_value = response.json::<Value>();
    assert_eq!(json_value["message"], "User created (v2)");
    assert_eq!(json_value["data"]["version"], "v2");
    assert_eq!(json_value["data"]["user"]["email"], "charlie@example.com");
    assert!(json_value["data"]["meta"]["created_at"].is_string());
}

#[tokio::test]
async fn test_v1_products_list() {
    let server = test_server();

    let response = server.get("/v1/products").await;

    response.assert_status_ok();

    let json_value = response.json::<Value>();
    assert_eq!(json_value["success"], true);
    assert_eq!(json_value["message"], "Products retrieved (v1)");
    assert_eq!(json_value["data"]["version"], "v1");
    assert_eq!(json_value["data"]["products"][0]["name"], "Laptop");
    assert_eq!(json_value["data"]["products"][0]["price"], 999.99);
}

#[tokio::test]
async fn test_v2_products_list() {
    let server = test_server();

    let response = server.get("/v2/products").await;

    response.assert_status_ok();

    let json_value = response.json::<Value>();
    assert_eq!(json_value["data"]["version"], "v2");
    assert_eq!(json_value["data"]["products"][0]["currency"], "USD");
    assert_eq!(json_value["data"]["products"][0]["stock"], 50);
    assert_eq!(json_value["data"]["products"][0]["category"], "Electronics");
}

#[tokio::test]
async fn test_v1_products_get_by_id() {
    let server = test_server();

    let response = server.get("/v1/products/1").await;

    response.assert_status_ok();

    let json_value = response.json::<Value>();
    assert_eq!(json_value["success"], true);
    assert_eq!(json_value["message"], "Product retrieved (v1)");
    assert_eq!(json_value["data"]["version"], "v1");
    assert_eq!(json_value["data"]["product"]["id"], 1);
    assert_eq!(json_value["data"]["product"]["name"], "Laptop");
    assert_eq!(json_value["data"]["product"]["price"], 999.99);
}

#[tokio::test]
async fn test_v2_products_get_by_id() {
    let server = test_server();

    let response = server.get("/v2/products/5").await;

    response.assert_status_ok();

    let json_value = response.json::<Value>();
    assert_eq!(json_value["data"]["version"], "v2");
    assert_eq!(json_value["data"]["product"]["id"], 5);
    assert_eq!(json_value["data"]["product"]["currency"], "USD");
    assert_eq!(
        json_value["data"]["product"]["description"],
        "High-performance laptop"
    );
}

#[tokio::test]
async fn test_different_versions_isolated() {
    let server = test_server();

    let v1_response = server.get("/v1/users").await;
    let v2_response = server.get("/v2/users").await;

    let v1_json = v1_response.json::<Value>();
    let v2_json = v2_response.json::<Value>();

    assert!(v1_json["data"]["users"][0]["email"].is_null());
    assert!(v1_json["data"]["meta"].is_null());

    assert_eq!(v2_json["data"]["users"][0]["email"], "alice@example.com");
    assert_eq!(v2_json["data"]["meta"]["total"], 2);
}

#[tokio::test]
async fn test_versioned_routes_path_isolation() {
    let server = test_server();

    let v1_users = server.get("/v1/users/123").await;
    let v2_users = server.get("/v2/users/123").await;
    let v1_products = server.get("/v1/products/456").await;
    let v2_products = server.get("/v2/products/456").await;

    v1_users.assert_status_ok();
    v2_users.assert_status_ok();
    v1_products.assert_status_ok();
    v2_products.assert_status_ok();

    assert_eq!(v1_users.json::<Value>()["data"]["version"], "v1");
    assert_eq!(v2_users.json::<Value>()["data"]["version"], "v2");
    assert_eq!(v1_products.json::<Value>()["data"]["version"], "v1");
    assert_eq!(v2_products.json::<Value>()["data"]["version"], "v2");
}
