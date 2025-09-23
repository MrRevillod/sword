use axum_test::{TestServer, multipart::MultipartForm};
use serde_json::Value;
use std::error::Error;
use sword::prelude::*;
use tokio::time;

#[controller("/test")]
struct TestController;

#[routes]
impl TestController {
    #[get("/timeout")]
    async fn timeout(&self, _: Context) -> HttpResult<HttpResponse> {
        time::sleep(time::Duration::from_secs(3)).await;
        Ok(HttpResponse::Ok().message("This should not be reached"))
    }

    #[get("/timeout-boundary")]
    async fn timeout_boundary(&self, _: Context) -> HttpResult<HttpResponse> {
        time::sleep(time::Duration::from_millis(2000)).await;
        Ok(HttpResponse::Ok().message("This should timeout"))
    }

    #[get("/timeout-just-under")]
    async fn timeout_just_under(&self, _: Context) -> HttpResult<HttpResponse> {
        time::sleep(time::Duration::from_millis(1900)).await;
        Ok(HttpResponse::Ok().message("This should complete"))
    }

    #[get("/timeout-just-over")]
    async fn timeout_just_over(&self, _: Context) -> HttpResult<HttpResponse> {
        time::sleep(time::Duration::from_millis(2100)).await;
        Ok(HttpResponse::Ok().message("This should timeout"))
    }

    #[get("/no-timeout")]
    async fn no_timeout(&self, _: Context) -> HttpResult<HttpResponse> {
        Ok(HttpResponse::Ok().message("Quick response"))
    }

    #[post("/content-type-json")]
    async fn content_type_json(&self, ctx: Context) -> HttpResult<HttpResponse> {
        let _body: Value = ctx.body()?;
        Ok(HttpResponse::Ok().message("JSON received"))
    }

    #[post("/content-type-form")]
    async fn content_type_form(&self, _: Context) -> HttpResult<HttpResponse> {
        Ok(HttpResponse::Ok().message("Form data received"))
    }

    #[post("/content-type-any")]
    async fn content_type_any(&self, ctx: Context) -> HttpResult<HttpResponse> {
        let _body: String = ctx.body()?;
        Ok(HttpResponse::Ok().message("Any content type"))
    }

    #[get("/no-body")]
    async fn no_body(&self, _: Context) -> HttpResult<HttpResponse> {
        Ok(HttpResponse::Ok().message("No body required"))
    }
}

#[tokio::test]
async fn timeout() -> Result<(), Box<dyn Error>> {
    let app = Application::builder()?
        .with_controller::<TestController>()
        .build();
    let test_app = TestServer::new(app.router())?;

    let response = test_app.get("/test/timeout").await;
    let json = response.json::<ResponseBody>();

    let expected = ResponseBody {
        code: 408,
        success: false,
        message: "Request Timeout".into(),
        data: Value::Null,
        timestamp: json.timestamp,
    };

    assert_eq!(json.code, expected.code);
    assert_eq!(json.success, expected.success);
    assert_eq!(json.message, expected.message);
    assert_eq!(json.data, expected.data);

    Ok(())
}

#[tokio::test]
async fn timeout_boundary_exact() -> Result<(), Box<dyn Error>> {
    let app = Application::builder()?
        .with_controller::<TestController>()
        .build();
    let test_app = TestServer::new(app.router())?;

    let response = test_app.get("/test/timeout-boundary").await;

    assert_eq!(response.status_code(), 408);

    let json = response.json::<ResponseBody>();
    assert_eq!(json.code, 408);
    assert!(!json.success);
    assert_eq!(json.message, "Request Timeout".into());

    Ok(())
}

#[tokio::test]
async fn timeout_just_under_limit() -> Result<(), Box<dyn Error>> {
    let app = Application::builder()?
        .with_controller::<TestController>()
        .build();
    let test_app = TestServer::new(app.router())?;

    let response = test_app.get("/test/timeout-just-under").await;
    assert_eq!(response.status_code(), 200);

    let json = response.json::<ResponseBody>();
    assert_eq!(json.code, 200);
    assert!(json.success);
    assert!(json.message.contains("This should complete"));

    Ok(())
}

#[tokio::test]
async fn timeout_just_over_limit() -> Result<(), Box<dyn Error>> {
    let app = Application::builder()?
        .with_controller::<TestController>()
        .build();
    let test_app = TestServer::new(app.router())?;

    let response = test_app.get("/test/timeout-just-over").await;

    assert_eq!(response.status_code(), 408);

    let json = response.json::<ResponseBody>();
    assert_eq!(json.code, 408);
    assert!(!json.success);
    assert_eq!(json.message, "Request Timeout".into());

    Ok(())
}

#[tokio::test]
async fn no_timeout_quick_response() -> Result<(), Box<dyn Error>> {
    let app = Application::builder()?
        .with_controller::<TestController>()
        .build();
    let test_app = TestServer::new(app.router())?;

    let response = test_app.get("/test/no-timeout").await;

    assert_eq!(response.status_code(), 200);

    let json = response.json::<ResponseBody>();

    assert_eq!(json.code, 200);
    assert!(json.success);
    assert!(json.message.contains("Quick response"));

    Ok(())
}

#[tokio::test]
async fn content_type_json_valid() -> Result<(), Box<dyn Error>> {
    let app = Application::builder()?
        .with_controller::<TestController>()
        .build();
    let test_app = TestServer::new(app.router())?;

    let response = test_app
        .post("/test/content-type-json")
        .json(&serde_json::json!({"test": "data"}))
        .await;

    let json = response.json::<ResponseBody>();

    assert_eq!(response.status_code(), 200);

    assert_eq!(json.code, 200);
    assert!(json.success);
    assert!(json.message.contains("JSON received"));

    Ok(())
}

#[tokio::test]
async fn content_type_multipart_valid() -> Result<(), Box<dyn Error>> {
    let app = Application::builder()?
        .with_controller::<TestController>()
        .build();
    let test_app = TestServer::new(app.router())?;

    let response = test_app
        .post("/test/content-type-form")
        .multipart(MultipartForm::new().add_text("field", "value"))
        .await;

    assert_eq!(response.status_code(), 200);

    let json = response.json::<ResponseBody>();
    assert_eq!(json.code, 200);
    assert!(json.success);
    assert!(json.message.contains("Form data received"));

    Ok(())
}

#[tokio::test]
async fn content_type_invalid() -> Result<(), Box<dyn Error>> {
    let app = Application::builder()?
        .with_controller::<TestController>()
        .build();
    let test_app = TestServer::new(app.router())?;

    let response = test_app
        .post("/test/content-type-any")
        .text("plain text data")
        .await;

    assert_eq!(response.status_code(), 415);

    let json = response.json::<ResponseBody>();
    assert_eq!(json.code, 415);
    assert!(!json.success);
    assert!(json.message.contains(
        "Only application/json and multipart/form-data content types are supported"
    ));

    Ok(())
}

#[tokio::test]
async fn content_type_xml_invalid() -> Result<(), Box<dyn Error>> {
    let app = Application::builder()?
        .with_controller::<TestController>()
        .build();
    let test_app = TestServer::new(app.router())?;

    use axum::body::Bytes;
    let response = test_app
        .post("/test/content-type-any")
        .bytes(Bytes::from("<xml>data</xml>"))
        .content_type("application/xml")
        .await;

    // Should reject XML content type
    assert_eq!(response.status_code(), 415);

    let json = response.json::<ResponseBody>();
    assert_eq!(json.code, 415);
    assert!(!json.success);
    assert!(json.message.contains(
        "Only application/json and multipart/form-data content types are supported"
    ));

    Ok(())
}

#[tokio::test]
async fn content_type_form_urlencoded_invalid() -> Result<(), Box<dyn Error>> {
    let app = Application::builder()?
        .with_controller::<TestController>()
        .build();
    let test_app = TestServer::new(app.router())?;

    use axum::body::Bytes;
    let response = test_app
        .post("/test/content-type-any")
        .bytes(Bytes::from("key=value&another=data"))
        .content_type("application/x-www-form-urlencoded")
        .await;

    // Should reject form-urlencoded content type
    assert_eq!(response.status_code(), 415);

    let json = response.json::<ResponseBody>();
    assert_eq!(json.code, 415);
    assert!(!json.success);
    assert!(json.message.contains(
        "Only application/json and multipart/form-data content types are supported"
    ));

    Ok(())
}

#[tokio::test]
async fn content_type_no_body_allowed() -> Result<(), Box<dyn Error>> {
    let app = Application::builder()?
        .with_controller::<TestController>()
        .build();
    let test_app = TestServer::new(app.router())?;

    // GET request with no body should pass content type check
    let response = test_app.get("/test/no-body").await;

    assert_eq!(response.status_code(), 200);

    let json = response.json::<ResponseBody>();
    assert_eq!(json.code, 200);
    assert!(json.success);
    assert!(json.message.contains("No body required"));

    Ok(())
}

#[tokio::test]
async fn content_type_missing_header_with_body() -> Result<(), Box<dyn Error>> {
    let app = Application::builder()?
        .with_controller::<TestController>()
        .build();
    let test_app = TestServer::new(app.router())?;

    let response = test_app
        .post("/test/content-type-any")
        .text("some data without content type")
        .await;

    assert_eq!(response.status_code(), 415);

    let json = response.json::<ResponseBody>();
    assert_eq!(json.code, 415);
    assert!(!json.success);
    assert!(json.message.contains(
        "Only application/json and multipart/form-data content types are supported"
    ));

    Ok(())
}

#[tokio::test]
async fn content_type_case_sensitivity() -> Result<(), Box<dyn Error>> {
    let app = Application::builder()?
        .with_controller::<TestController>()
        .build();
    let test_app = TestServer::new(app.router())?;

    use axum::body::Bytes;
    let response = test_app
        .post("/test/content-type-json")
        .bytes(Bytes::from(r#"{"test": "data"}"#))
        .content_type("Application/JSON")
        .await;

    assert_eq!(response.status_code(), 415);

    Ok(())
}

#[tokio::test]
async fn content_type_json_with_charset() -> Result<(), Box<dyn Error>> {
    let app = Application::builder()?
        .with_controller::<TestController>()
        .build();
    let test_app = TestServer::new(app.router())?;

    use axum::body::Bytes;
    let response = test_app
        .post("/test/content-type-json")
        .bytes(Bytes::from(r#"{"test": "data"}"#))
        .content_type("application/json; charset=utf-8")
        .await;

    assert_eq!(response.status_code(), 415);

    let json = response.json::<ResponseBody>();
    assert_eq!(json.code, 415);
    assert!(!json.success);

    Ok(())
}
