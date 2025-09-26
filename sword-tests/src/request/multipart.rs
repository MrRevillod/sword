use axum_test::{
    TestServer,
    multipart::{MultipartForm, Part},
};

use std::fs;
use sword::prelude::*;

use crate::utils::TempFile;

#[controller("/")]
struct TestController {}

#[routes]
impl TestController {
    #[post("/multipart")]
    async fn hello(&self, ctx: Context) -> HttpResult<HttpResponse> {
        let mut fields = vec![];
        let mut multipart = ctx.multipart().await?;

        while let Some(field) = multipart.next_field().await.map_err(|e| {
            eprintln!("Error reading multipart field: {}", e);
            HttpResponse::BadRequest().message("Failed to read multipart field")
        })? {
            let name = field.name().unwrap_or("Unnamed").to_string();
            let file_name = field.file_name().unwrap_or("No file name").to_string();

            let content_type = field
                .content_type()
                .map(|ct| ct.to_string())
                .unwrap_or("No content type".to_string());

            let data = field.bytes().await.unwrap();

            fields.push(serde_json::json!({
                "name": name,
                "file_name": file_name,
                "content_type": content_type,
                "data_length": data.len(),
            }));
        }

        Ok(HttpResponse::Ok().data(fields).message("Hello, Multipart!"))
    }
}

#[tokio::test]
async fn exceed_limit() -> Result<(), Box<dyn std::error::Error>> {
    let app = Application::builder()
        .with_controller::<TestController>()
        .build();

    let test = TestServer::new(app.router()).unwrap();

    let temp_file = TempFile::with_size(1024 * 1024 * 2); // 2 MB
    let bytes = fs::read(&temp_file.path).expect("Failed to read test file");

    let part = Part::bytes(bytes)
        .file_name("large_test_file.txt")
        .mime_type("text/plain");

    let form = MultipartForm::new()
        .add_text("field1", "value1")
        .add_part("file", part);

    let response = test.post("/multipart").multipart(form).await;
    let json = response.json::<ResponseBody>();

    assert_eq!(response.status_code(), 413);

    assert_eq!(json.message, "Request payload too large".into());
    assert_eq!(json.data["type"], "PayloadTooLarge");

    Ok(())
}

/// Tests that a file exactly at the body limit (considering multipart overhead) is accepted.
/// The effective limit is ~975KB due to multipart headers/boundaries overhead.
#[tokio::test]
async fn body_limit_exactly_at_limit() -> Result<(), Box<dyn std::error::Error>> {
    let app = Application::builder()
        .with_controller::<TestController>()
        .build();

    let test = TestServer::new(app.router()).unwrap();

    let temp_file = TempFile::with_size(975 * 1024);
    let bytes = fs::read(&temp_file.path).expect("Failed to read test file");

    let part = Part::bytes(bytes)
        .file_name("exactly_at_limit_file.txt")
        .mime_type("text/plain");

    let form = MultipartForm::new()
        .add_text("field1", "value1")
        .add_part("file", part);

    let response = test.post("/multipart").multipart(form).await;

    assert_eq!(response.status_code(), 200);
    let json = response.json::<ResponseBody>();
    assert_eq!(json.message, "Hello, Multipart!".into());

    Ok(())
}

#[tokio::test]
async fn body_limit_just_under_limit() -> Result<(), Box<dyn std::error::Error>> {
    let app = Application::builder()
        .with_controller::<TestController>()
        .build();

    let test = TestServer::new(app.router()).unwrap();

    let temp_file = TempFile::with_size(970 * 1024);
    let bytes = fs::read(&temp_file.path).expect("Failed to read test file");

    let part = Part::bytes(bytes)
        .file_name("just_under_limit_file.txt")
        .mime_type("text/plain");

    let form = MultipartForm::new()
        .add_text("field1", "value1")
        .add_part("file", part);

    let response = test.post("/multipart").multipart(form).await;

    assert_eq!(response.status_code(), 200);
    let json = response.json::<ResponseBody>();
    assert_eq!(json.message, "Hello, Multipart!".into());

    Ok(())
}

#[tokio::test]
async fn body_limit_just_over_limit() -> Result<(), Box<dyn std::error::Error>> {
    let app = Application::builder()
        .with_controller::<TestController>()
        .build();

    let test = TestServer::new(app.router()).unwrap();

    let temp_file = TempFile::with_size(976 * 1024);
    let bytes = fs::read(&temp_file.path).expect("Failed to read test file");

    let part = Part::bytes(bytes)
        .file_name("just_over_limit_file.txt")
        .mime_type("text/plain");

    let form = MultipartForm::new()
        .add_text("field1", "value1")
        .add_part("file", part);

    let response = test.post("/multipart").multipart(form).await;

    assert_eq!(response.status_code(), 413);
    let json = response.json::<ResponseBody>();
    assert_eq!(json.message, "Request payload too large".into());
    assert_eq!(json.data["type"], "PayloadTooLarge");

    Ok(())
}

#[tokio::test]
async fn body_limit_multiple_fields_exceed_limit()
-> Result<(), Box<dyn std::error::Error>> {
    let app = Application::builder()
        .with_controller::<TestController>()
        .build();

    let test = TestServer::new(app.router()).unwrap();

    // Create multiple smaller files that together exceed the limit
    let temp_file1 = TempFile::with_size(700 * 1024); // 700 KB
    let temp_file2 = TempFile::with_size(700 * 1024); // 700 KB
    // Total: ~1.4 MB plus multipart overhead should exceed 1MB limit

    let bytes1 = fs::read(&temp_file1.path).expect("Failed to read test file 1");
    let bytes2 = fs::read(&temp_file2.path).expect("Failed to read test file 2");

    let part1 = Part::bytes(bytes1)
        .file_name("file1.txt")
        .mime_type("text/plain");

    let part2 = Part::bytes(bytes2)
        .file_name("file2.txt")
        .mime_type("text/plain");

    let form = MultipartForm::new()
        .add_text("field1", "value1")
        .add_part("file1", part1)
        .add_part("file2", part2);

    let response = test.post("/multipart").multipart(form).await;

    assert_eq!(response.status_code(), 413);
    let json = response.json::<ResponseBody>();
    assert_eq!(json.message, "Request payload too large".into());
    assert_eq!(json.data["type"], "PayloadTooLarge");

    Ok(())
}

#[tokio::test]
async fn body_limit_small_fields_within_limit()
-> Result<(), Box<dyn std::error::Error>> {
    let app = Application::builder()
        .with_controller::<TestController>()
        .build();

    let test = TestServer::new(app.router()).unwrap();

    // Create multiple smaller files that together stay within the limit
    let temp_file1 = TempFile::with_size(300 * 1024); // 300 KB
    let temp_file2 = TempFile::with_size(300 * 1024); // 300 KB
    // Total: ~600 KB plus multipart overhead should be within 1MB limit

    let bytes1 = fs::read(&temp_file1.path).expect("Failed to read test file 1");
    let bytes2 = fs::read(&temp_file2.path).expect("Failed to read test file 2");

    let part1 = Part::bytes(bytes1)
        .file_name("file1.txt")
        .mime_type("text/plain");

    let part2 = Part::bytes(bytes2)
        .file_name("file2.txt")
        .mime_type("text/plain");

    let form = MultipartForm::new()
        .add_text("field1", "value1")
        .add_text("field2", "value2")
        .add_part("file1", part1)
        .add_part("file2", part2);

    let response = test.post("/multipart").multipart(form).await;

    assert_eq!(response.status_code(), 200);
    let json = response.json::<ResponseBody>();

    assert_eq!(json.message, "Hello, Multipart!".into());

    Ok(())
}
