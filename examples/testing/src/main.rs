use serde_json::json;
use sword::prelude::*;

#[controller("/users")]
pub struct UsersController {}

#[routes]
impl UsersController {
    #[get("/")]
    async fn list_users(&self) -> HttpResponse {
        let data = json!({
            "users": [
                {"id": 1, "name": "Alice"},
                {"id": 2, "name": "Bob"}
            ]
        });

        HttpResponse::Ok().data(data)
    }
}

#[tokio::test]
async fn test_list_users() {
    use axum_test::TestServer;

    let app = Application::builder()
        .with_controller::<UsersController>()
        .build();

    let server = TestServer::new(app.router()).unwrap();
    let response = server.get("/users").await;
    let json = response.json::<ResponseBody>();

    assert_eq!(response.status_code(), 200);
    assert!(json.data.is_some());

    let data = json.data.unwrap();

    assert_eq!(
        data,
        json!({
            "users": [
                {"id": 1, "name": "Alice"},
                {"id": 2, "name": "Bob"}
            ]
        })
    );
}

fn main() {}
