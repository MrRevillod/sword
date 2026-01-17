use axum_test::{TestServer, http::StatusCode};
use serde::{Deserialize, Serialize};
use sword::prelude::*;

fn test_server() -> TestServer {
    let app = Application::builder()
        .with_module::<ExtractorModule>()
        .build();

    TestServer::new(app.router()).unwrap()
}

#[derive(Serialize)]
struct Mixed {
    path: UserIdPath,
    body: CreateUserDto,
}

#[derive(Serialize)]
struct Combined {
    path: UserIdPath,
    query: Option<SearchParams>,
    body: CreateUserDto,
}

// ===== DTOs =====

#[derive(Debug, Deserialize, Serialize, PartialEq)]
struct CreateUserDto {
    name: String,
    email: String,
    age: u32,
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
struct SearchParams {
    q: Option<String>,
    page: Option<u32>,
    limit: Option<u32>,
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
struct UserIdPath {
    id: u32,
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
struct PostPath {
    user_id: u32,
    post_id: u32,
}

// ===== Controller =====

#[controller("/extractors")]
pub struct ExtractorController {}

impl ExtractorController {
    #[post("/json")]
    async fn test_json(&self, Json(data): Json<CreateUserDto>) -> JsonResponse {
        JsonResponse::Ok().data(data)
    }

    #[get("/query")]
    async fn test_query(&self, Query(params): Query<SearchParams>) -> HttpResult {
        Ok(JsonResponse::Ok().data(params))
    }

    #[get("/path/{id}")]
    async fn test_path_single(
        &self,
        PathParams(path): PathParams<UserIdPath>,
    ) -> HttpResult {
        Ok(JsonResponse::Ok().data(path))
    }

    #[get("/path/{user_id}/posts/{post_id}")]
    async fn test_path_multiple(
        &self,
        PathParams(path): PathParams<PostPath>,
    ) -> JsonResponse {
        JsonResponse::Ok().data(path)
    }

    #[post("/combined/{id}")]
    async fn test_combined(
        &self,
        PathParams(path): PathParams<UserIdPath>,
        Query(params): Query<SearchParams>,
        Json(data): Json<CreateUserDto>,
    ) -> JsonResponse {
        JsonResponse::Ok().data(Combined {
            path,
            query: params,
            body: data,
        })
    }

    #[get("/request")]
    async fn test_request(&self, req: Request) -> HttpResult {
        let query: Option<SearchParams> = req.query()?;
        Ok(JsonResponse::Ok().data(query))
    }

    #[post("/mixed/{id}")]
    async fn test_mixed(
        &self,
        PathParams(path): PathParams<UserIdPath>,
        Json(data): Json<CreateUserDto>,
    ) -> JsonResponse {
        JsonResponse::Ok().data(Mixed { path, body: data })
    }

    #[get("/method")]
    async fn test_method(&self, method: Method) -> JsonResponse {
        JsonResponse::Ok().data(method.as_str())
    }

    #[get("/uri")]
    async fn test_uri(&self, uri: Uri) -> JsonResponse {
        JsonResponse::Ok().data(uri.path())
    }

    #[get("/headers")]
    async fn test_headers(&self, headers: Headers) -> JsonResponse {
        let user_agent = headers
            .get("user-agent")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("unknown");
        JsonResponse::Ok().data(user_agent)
    }

    #[post("/bytes")]
    async fn test_bytes(&self, bytes: Bytes) -> JsonResponse {
        let size = bytes.len();
        JsonResponse::Ok().data(size)
    }

    #[get("/combined-axum")]
    async fn test_combined_axum(
        &self,
        method: Method,
        uri: Uri,
        headers: Headers,
    ) -> JsonResponse {
        #[derive(Serialize)]
        struct AxumData {
            method: String,
            path: String,
            has_user_agent: bool,
        }

        JsonResponse::Ok().data(AxumData {
            method: method.as_str().to_string(),
            path: uri.path().to_string(),
            has_user_agent: headers.contains_key("user-agent"),
        })
    }
}

struct ExtractorModule;

impl Module for ExtractorModule {
    fn register_adapters(adapters: &AdapterRegistry) {
        adapters.register::<ExtractorController>();
    }
}

// ===== Tests =====

#[tokio::test]
async fn test_json_extractor_success() {
    let app = test_server();

    let user = CreateUserDto {
        name: "John Doe".to_string(),
        email: "john@example.com".to_string(),
        age: 30,
    };

    let response = app.post("/extractors/json").json(&user).await;

    response.assert_status_ok();

    let json = response.json::<JsonResponseBody>();
    let returned: CreateUserDto =
        serde_json::from_value(json.data.unwrap()).unwrap();
    assert_eq!(returned, user);
}

#[tokio::test]
async fn test_json_extractor_invalid_content_type() {
    let app = test_server();

    let response = app.post("/extractors/json").text("not json").await;

    response.assert_status(StatusCode::UNSUPPORTED_MEDIA_TYPE);
}

#[tokio::test]
async fn test_json_extractor_empty_body() {
    let app = test_server();

    let response = app
        .post("/extractors/json")
        .add_header("content-type", "application/json")
        .await;

    response.assert_status_bad_request();
}

#[tokio::test]
async fn test_query_extractor_with_params() {
    let app = test_server();

    let response = app.get("/extractors/query?q=test&page=1&limit=10").await;

    response.assert_status_ok();

    let json = response.json::<JsonResponseBody>();
    let returned: Option<SearchParams> =
        serde_json::from_value(json.data.unwrap()).unwrap();
    assert!(returned.is_some());

    let params = returned.unwrap();
    assert_eq!(params.q, Some("test".to_string()));
    assert_eq!(params.page, Some(1));
    assert_eq!(params.limit, Some(10));
}

#[tokio::test]
async fn test_query_extractor_without_params() {
    let app = test_server();

    let response = app.get("/extractors/query").await;

    response.assert_status_ok();

    let json = response.json::<JsonResponseBody>();
    if let Some(data) = json.data {
        let returned: Option<SearchParams> = serde_json::from_value(data).unwrap();
        assert!(returned.is_none());
    }
}

#[tokio::test]
async fn test_query_extractor_partial_params() {
    let app = test_server();

    let response = app.get("/extractors/query?q=test").await;

    response.assert_status_ok();

    let json = response.json::<JsonResponseBody>();
    let returned: Option<SearchParams> =
        serde_json::from_value(json.data.unwrap()).unwrap();
    assert!(returned.is_some());

    let params = returned.unwrap();
    assert_eq!(params.q, Some("test".to_string()));
    assert_eq!(params.page, None);
    assert_eq!(params.limit, None);
}

#[tokio::test]
async fn test_path_extractor_single_param() {
    let app = test_server();

    let response = app.get("/extractors/path/42").await;

    response.assert_status_ok();

    let json = response.json::<JsonResponseBody>();
    let returned: UserIdPath = serde_json::from_value(json.data.unwrap()).unwrap();
    assert_eq!(returned.id, 42);
}

#[tokio::test]
async fn test_path_extractor_multiple_params() {
    let app = test_server();

    let response = app.get("/extractors/path/10/posts/20").await;

    response.assert_status_ok();

    let json = response.json::<JsonResponseBody>();
    let returned: PostPath = serde_json::from_value(json.data.unwrap()).unwrap();
    assert_eq!(returned.user_id, 10);
    assert_eq!(returned.post_id, 20);
}

#[tokio::test]
async fn test_path_extractor_invalid_type() {
    let app = test_server();

    let response = app.get("/extractors/path/not_a_number").await;

    response.assert_status_bad_request();
}

#[tokio::test]
async fn test_combined_extractors() {
    let app = test_server();

    let user = CreateUserDto {
        name: "Jane Doe".to_string(),
        email: "jane@example.com".to_string(),
        age: 25,
    };

    let response = app
        .post("/extractors/combined/42?q=search&page=2")
        .json(&user)
        .await;

    response.assert_status_ok();

    #[derive(Deserialize)]
    struct Combined {
        path: UserIdPath,
        query: Option<SearchParams>,
        body: CreateUserDto,
    }

    let json = response.json::<JsonResponseBody>();
    let returned: Combined = serde_json::from_value(json.data.unwrap()).unwrap();
    assert_eq!(returned.path.id, 42);
    assert!(returned.query.is_some());
    assert_eq!(
        returned.query.as_ref().unwrap().q,
        Some("search".to_string())
    );
    assert_eq!(returned.query.as_ref().unwrap().page, Some(2));
    assert_eq!(returned.body, user);
}

#[tokio::test]
async fn test_backward_compatibility_with_request() {
    let app = test_server();

    let response = app.get("/extractors/request?q=test&page=1").await;

    response.assert_status_ok();

    let json = response.json::<JsonResponseBody>();
    let returned: Option<SearchParams> =
        serde_json::from_value(json.data.unwrap()).unwrap();
    assert!(returned.is_some());
}

#[tokio::test]
async fn test_mixed_extractors_and_request() {
    let app = test_server();

    let user = CreateUserDto {
        name: "Mixed Test".to_string(),
        email: "mixed@example.com".to_string(),
        age: 28,
    };

    let response = app.post("/extractors/mixed/99").json(&user).await;

    response.assert_status_ok();

    #[derive(Deserialize)]
    struct Mixed {
        path: UserIdPath,
        body: CreateUserDto,
    }

    let json = response.json::<JsonResponseBody>();
    let returned: Mixed = serde_json::from_value(json.data.unwrap()).unwrap();
    assert_eq!(returned.path.id, 99);
    assert_eq!(returned.body, user);
}

#[tokio::test]
async fn test_method_extractor() {
    let app = test_server();

    let response = app.get("/extractors/method").await;

    response.assert_status_ok();

    let json = response.json::<JsonResponseBody>();
    let method: String = serde_json::from_value(json.data.unwrap()).unwrap();
    assert_eq!(method, "GET");
}

#[tokio::test]
async fn test_uri_extractor() {
    let app = test_server();

    let response = app.get("/extractors/uri").await;

    response.assert_status_ok();

    let json = response.json::<JsonResponseBody>();
    let path: String = serde_json::from_value(json.data.unwrap()).unwrap();
    assert_eq!(path, "/uri");
}

#[tokio::test]
async fn test_headers_extractor() {
    let app = test_server();

    let response = app
        .get("/extractors/headers")
        .add_header("user-agent", "test-client/1.0")
        .await;

    response.assert_status_ok();

    let json = response.json::<JsonResponseBody>();
    let user_agent: String = serde_json::from_value(json.data.unwrap()).unwrap();
    assert_eq!(user_agent, "test-client/1.0");
}

#[tokio::test]
async fn test_bytes_extractor() {
    let app = test_server();

    let data = "Hello, World!";
    let response = app.post("/extractors/bytes").text(data).await;

    response.assert_status_ok();

    let json = response.json::<JsonResponseBody>();
    let size: usize = serde_json::from_value(json.data.unwrap()).unwrap();
    assert_eq!(size, data.len());
}

#[tokio::test]
async fn test_combined_axum_extractors() {
    let app = test_server();

    let response = app
        .get("/extractors/combined-axum")
        .add_header("user-agent", "axum-test")
        .await;

    response.assert_status_ok();

    #[derive(Deserialize)]
    struct AxumData {
        method: String,
        path: String,
        has_user_agent: bool,
    }

    let json = response.json::<JsonResponseBody>();
    let data: AxumData = serde_json::from_value(json.data.unwrap()).unwrap();
    assert_eq!(data.method, "GET");
    assert_eq!(data.path, "/combined-axum");
    assert!(data.has_user_agent);
}
