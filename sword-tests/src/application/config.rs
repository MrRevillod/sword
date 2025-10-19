use std::process::Command;

use axum_test::TestServer;
use serde::{Deserialize, Serialize};
use sword::prelude::*;

#[derive(Clone, Serialize, Deserialize)]
#[config(key = "my-custom-section")]
struct MyConfig {
    custom_key: String,
    env_user: String,
}

#[controller("/test")]
struct TestController {
    custom_config: MyConfig,
}

#[routes]
impl TestController {
    #[get("/hello")]
    async fn hello(&self) -> HttpResponse {
        HttpResponse::Ok()
            .data(&self.custom_config)
            .message("Test controller response")
    }
}

#[tokio::test]
async fn test_application() {
    let app = Application::builder()
        .with_controller::<TestController>()
        .build();

    let test = TestServer::new(app.router()).unwrap();

    let response = test.get("/test/hello").await;
    let json_body = response.json::<ResponseBody>();

    assert_eq!(response.status_code(), 200);
    assert!(json_body.data.is_some());

    let data = json_body.data.unwrap();

    let expected = MyConfig {
        custom_key: "value".to_string(),
        env_user: Command::new("sh")
            .arg("-c")
            .arg("echo $USER")
            .output()
            .expect("Failed to get environment variable")
            .stdout
            .into_iter()
            .map(|b| b as char)
            .collect::<String>()
            .trim()
            .to_string(),
    };

    assert_eq!(data["custom_key"], expected.custom_key);
}
