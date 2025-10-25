use axum_test::TestServer;
use serde_json::json;
use sword::prelude::*;
use tower_http::cors::CorsLayer;

#[allow(unused)]
#[derive(Debug, Clone)]
pub enum LogLevel {
    Debug,
    Info,
    Warning,
    Error,
}

#[derive(Debug, Clone)]
#[allow(unused)]
pub enum DatabaseConfig {
    Sqlite(String),
    Postgres { host: String, port: u16 },
    Memory,
}

#[derive(Debug, Clone)]
#[allow(unused)]
pub enum AuthMethod {
    Basic { username: String, password: String },
    None,
}

#[middleware]
pub struct FileValidationMiddleware;

impl OnRequestWithConfig<(&'static str, &'static str)> for FileValidationMiddleware {
    async fn on_request_with_config(
        &self,
        config: (&'static str, &'static str),
        mut req: Request,
        next: Next,
    ) -> MiddlewareResult {
        req.extensions
            .insert((config.0.to_string(), config.1.to_string()));

        req.run(next).await
    }
}

#[middleware]
struct ExtensionsTestMiddleware;

impl OnRequest for ExtensionsTestMiddleware {
    async fn on_request(&self, mut req: Request, next: Next) -> MiddlewareResult {
        req.extensions
            .insert::<String>("test_extension".to_string());

        req.run(next).await
    }
}

#[middleware]
struct MwWithState;

impl OnRequest for MwWithState {
    async fn on_request(&self, mut req: Request, next: Next) -> MiddlewareResult {
        req.extensions.insert::<u16>(8080);
        req.run(next).await
    }
}

#[middleware]
struct RoleMiddleware;

impl OnRequestWithConfig<Vec<&'static str>> for RoleMiddleware {
    async fn on_request_with_config(
        &self,
        roles: Vec<&'static str>,
        mut req: Request,
        next: Next,
    ) -> MiddlewareResult {
        let roles_owned: Vec<String> = roles.iter().map(|s| s.to_string()).collect();
        req.extensions.insert(roles_owned);

        req.run(next).await
    }
}

#[middleware]
struct TupleConfigMiddleware;

impl OnRequestWithConfig<(&'static str, &'static str)> for TupleConfigMiddleware {
    async fn on_request_with_config(
        &self,
        config: (&'static str, &'static str),
        mut req: Request,
        next: Next,
    ) -> MiddlewareResult {
        req.extensions
            .insert((config.0.to_string(), config.1.to_string()));
        req.run(next).await
    }
}

#[middleware]
struct ArrayConfigMiddleware;

impl OnRequestWithConfig<[i32; 3]> for ArrayConfigMiddleware {
    async fn on_request_with_config(
        &self,
        config: [i32; 3],
        mut req: Request,
        next: Next,
    ) -> MiddlewareResult {
        req.extensions.insert(config);

        req.run(next).await
    }
}

#[middleware]
struct StringConfigMiddleware;

impl OnRequestWithConfig<String> for StringConfigMiddleware {
    async fn on_request_with_config(
        &self,
        config: String,
        mut req: Request,
        next: Next,
    ) -> MiddlewareResult {
        req.extensions.insert(config);

        req.run(next).await
    }
}

#[middleware]
struct StrConfigMiddleware;

impl OnRequestWithConfig<&'static str> for StrConfigMiddleware {
    async fn on_request_with_config(
        &self,
        config: &'static str,
        mut req: Request,
        next: Next,
    ) -> MiddlewareResult {
        req.extensions.insert(config.to_string());

        req.run(next).await
    }
}

#[middleware]
struct NumberConfigMiddleware;

impl OnRequestWithConfig<i32> for NumberConfigMiddleware {
    async fn on_request_with_config(
        &self,
        config: i32,
        mut req: Request,
        next: Next,
    ) -> MiddlewareResult {
        req.extensions.insert(config);

        req.run(next).await
    }
}

#[middleware]
struct BoolConfigMiddleware;

impl OnRequestWithConfig<bool> for BoolConfigMiddleware {
    async fn on_request_with_config(
        &self,
        config: bool,
        mut req: Request,
        next: Next,
    ) -> MiddlewareResult {
        req.extensions.insert(config);

        req.run(next).await
    }
}

#[middleware]
struct ComplexConfigMiddleware;

impl OnRequestWithConfig<(Vec<&'static str>, i32, bool)>
    for ComplexConfigMiddleware
{
    async fn on_request_with_config(
        &self,
        config: (Vec<&'static str>, i32, bool),
        mut req: Request,
        next: Next,
    ) -> MiddlewareResult {
        let owned_config = (
            config
                .0
                .iter()
                .map(|s| s.to_string())
                .collect::<Vec<String>>(),
            config.1,
            config.2,
        );

        req.extensions.insert(owned_config);

        req.run(next).await
    }
}

#[middleware]
struct FunctionConfigMiddleware;

impl OnRequestWithConfig<Vec<String>> for FunctionConfigMiddleware {
    async fn on_request_with_config(
        &self,
        config: Vec<String>,
        mut req: Request,
        next: Next,
    ) -> MiddlewareResult {
        req.extensions.insert(config);

        req.run(next).await
    }
}

#[middleware]
struct MathConfigMiddleware;

impl OnRequestWithConfig<i32> for MathConfigMiddleware {
    async fn on_request_with_config(
        &self,
        config: i32,
        mut req: Request,
        next: Next,
    ) -> MiddlewareResult {
        req.extensions.insert(config);

        req.run(next).await
    }
}

#[middleware]
struct ConstConfigMiddleware;

impl OnRequestWithConfig<&'static str> for ConstConfigMiddleware {
    async fn on_request_with_config(
        &self,
        config: &'static str,
        mut req: Request,
        next: Next,
    ) -> MiddlewareResult {
        req.extensions.insert(config.to_string());

        req.run(next).await
    }
}

#[middleware]
struct LogMiddleware;

impl OnRequestWithConfig<LogLevel> for LogMiddleware {
    async fn on_request_with_config(
        &self,
        config: LogLevel,
        mut req: Request,
        next: Next,
    ) -> MiddlewareResult {
        req.extensions.insert(config);

        req.run(next).await
    }
}

#[middleware]
struct DatabaseMiddleware;

impl OnRequestWithConfig<DatabaseConfig> for DatabaseMiddleware {
    async fn on_request_with_config(
        &self,
        config: DatabaseConfig,
        mut req: Request,
        next: Next,
    ) -> MiddlewareResult {
        req.extensions.insert(config);

        req.run(next).await
    }
}

#[middleware]
struct AuthMiddleware;

impl OnRequestWithConfig<AuthMethod> for AuthMiddleware {
    async fn on_request_with_config(
        &self,
        config: AuthMethod,
        mut req: Request,
        next: Next,
    ) -> MiddlewareResult {
        req.extensions.insert(config);

        req.run(next).await
    }
}

#[middleware]
struct EnumOptionMiddleware;

impl OnRequestWithConfig<Option<LogLevel>> for EnumOptionMiddleware {
    async fn on_request_with_config(
        &self,
        config: Option<LogLevel>,
        mut req: Request,
        next: Next,
    ) -> MiddlewareResult {
        req.extensions.insert(config);

        req.run(next).await
    }
}

#[middleware]
struct EnumVecMiddleware;

impl OnRequestWithConfig<Vec<LogLevel>> for EnumVecMiddleware {
    async fn on_request_with_config(
        &self,
        config: Vec<LogLevel>,
        mut req: Request,
        next: Next,
    ) -> MiddlewareResult {
        req.extensions.insert(config);

        req.run(next).await
    }
}

#[controller("/test")]
struct TestController {}

#[routes]
impl TestController {
    #[get("/extensions-test")]
    #[uses(ExtensionsTestMiddleware)]
    async fn extensions_test(&self, req: Request) -> HttpResponse {
        let extension_value = req.extensions.get::<String>();

        HttpResponse::Ok()
            .message("Test controller response with extensions")
            .data(json!({
                "extension_value": extension_value.cloned().unwrap_or_default()
            }))
    }

    #[get("/middleware-state")]
    #[uses(ExtensionsTestMiddleware)]
    #[uses(MwWithState)]
    async fn middleware_state(&self, req: Request) -> HttpResult {
        let port = req.extensions.get::<u16>().cloned().unwrap_or(0);
        let message = req.extensions.get::<String>().cloned().unwrap_or_default();

        let json = json!({
            "port": port,
            "message": message
        });

        Ok(HttpResponse::Ok()
            .message("Test controller response with middleware state")
            .data(json))
    }

    #[get("/role-test")]
    #[uses(RoleMiddleware, config = vec!["admin", "user"])]
    async fn role_test(&self, req: Request) -> HttpResponse {
        let config = req
            .extensions
            .get::<Vec<String>>()
            .cloned()
            .unwrap_or_default();

        HttpResponse::Ok().data(json!({
            "roles": config
        }))
    }

    #[get("/error-test")]
    #[uses(FileValidationMiddleware, config = ("jpg", "png"))]
    async fn error_test(&self, req: Request) -> HttpResponse {
        let config = req
            .extensions
            .get::<(String, String)>()
            .cloned()
            .unwrap_or(("".to_string(), "".to_string()));

        HttpResponse::Ok().data(json!({
            "allowed_formats": [config.0, config.1]
        }))
    }

    #[get("/tower-middleware-test")]
    #[uses(CorsLayer::permissive())]
    async fn tower_middleware_test(&self) -> HttpResponse {
        HttpResponse::Ok()
            .message("Test with tower middleware")
            .data(json!({"middleware": "cors"}))
    }

    #[get("/tuple-config-test")]
    #[uses(TupleConfigMiddleware, config = ("jpg", "png"))]
    async fn tuple_config_test(&self, req: Request) -> HttpResponse {
        let config = req
            .extensions
            .get::<(String, String)>()
            .cloned()
            .unwrap_or(("".to_string(), "".to_string()));

        HttpResponse::Ok().message("Tuple config test").data(json!({
            "config_type": "tuple",
            "config": [config.0, config.1]
        }))
    }

    #[get("/array-config-test")]
    #[uses(ArrayConfigMiddleware, config = [1, 2, 3])]
    async fn array_config_test(&self, req: Request) -> HttpResponse {
        let config = req
            .extensions
            .get::<[i32; 3]>()
            .cloned()
            .unwrap_or([0, 0, 0]);

        HttpResponse::Ok().message("Array config test").data(json!({
            "config_type": "array",
            "config": config
        }))
    }

    #[get("/string-config-test")]
    #[uses(StringConfigMiddleware, config = "test string".to_string())]
    async fn string_config_test(&self, req: Request) -> HttpResponse {
        let config = req.extensions.get::<String>().cloned().unwrap_or_default();

        HttpResponse::Ok()
            .message("String config test")
            .data(json!({
                "config_type": "string",
                "config": config
            }))
    }

    #[get("/str-config-test")]
    #[uses(StrConfigMiddleware, config = "test str")]
    async fn str_config_test(&self, req: Request) -> HttpResponse {
        let config = req.extensions.get::<String>().cloned().unwrap_or_default();

        HttpResponse::Ok().message("Str config test").data(json!({
            "config_type": "str",
            "config": config
        }))
    }

    #[get("/number-config-test")]
    #[uses(NumberConfigMiddleware, config = 42)]
    async fn number_config_test(&self, req: Request) -> HttpResponse {
        let config = req.extensions.get::<i32>().cloned().unwrap_or(0);

        HttpResponse::Ok()
            .message("Number config test")
            .data(json!({
                "config_type": "number",
                "config": config
            }))
    }

    #[get("/bool-config-test")]
    #[uses(BoolConfigMiddleware, config = true)]
    async fn bool_config_test(&self, req: Request) -> HttpResponse {
        let config = req.extensions.get::<bool>().cloned().unwrap_or(false);

        HttpResponse::Ok().message("Bool config test").data(json!({
            "config_type": "bool",
            "config": config
        }))
    }
}

#[tokio::test]
async fn extensions_mw_test() {
    let app = Application::builder()
        .with_controller::<TestController>()
        .build();

    let test = TestServer::new(app.router()).unwrap();
    let response = test.get("/test/extensions-test").await;
    assert_eq!(response.status_code(), 200);

    let json = response.json::<ResponseBody>();
    assert!(json.data.is_some());

    let data = json.data.unwrap();
    assert_eq!(data["extension_value"], "test_extension");
}

#[tokio::test]
async fn middleware_state() {
    let app = Application::builder()
        .with_controller::<TestController>()
        .build();

    let test = TestServer::new(app.router()).unwrap();
    let response = test.get("/test/middleware-state").await;

    assert_eq!(response.status_code(), 200);

    let json = response.json::<ResponseBody>();
    assert!(json.data.is_some());

    let data = json.data.unwrap();
    assert_eq!(data["port"], 8080);
    assert_eq!(data["message"], "test_extension");
}

#[tokio::test]
async fn role_middleware_test() {
    let app = Application::builder()
        .with_controller::<TestController>()
        .build();

    let test = TestServer::new(app.router()).unwrap();
    let response = test.get("/test/role-test").await;

    assert_eq!(response.status_code(), 200);

    let json = response.json::<ResponseBody>();
    assert!(json.data.is_some());

    let data = json.data.unwrap();
    assert_eq!(data["roles"], json!(["admin", "user"]));
}

#[tokio::test]
async fn error_middleware_test() {
    let app = Application::builder()
        .with_controller::<TestController>()
        .build();

    let test = TestServer::new(app.router()).unwrap();
    let response = test.get("/test/error-test").await;

    assert_eq!(response.status_code(), 200);

    let json = response.json::<ResponseBody>();
    assert!(json.data.is_some());

    let data = json.data.unwrap();
    assert_eq!(data["allowed_formats"], json!(["jpg", "png"]));
}

#[tokio::test]
async fn tower_middleware_test() {
    let app = Application::builder()
        .with_controller::<TestController>()
        .build();

    let test = TestServer::new(app.router()).unwrap();
    let response = test.get("/test/tower-middleware-test").await;

    assert_eq!(response.status_code(), 200);

    let json = response.json::<ResponseBody>();
    assert!(json.data.is_some());

    let data = json.data.unwrap();
    assert_eq!(data["middleware"], "cors");
}

#[tokio::test]
async fn tuple_config_middleware_test() {
    let app = Application::builder()
        .with_controller::<TestController>()
        .build();

    let test = TestServer::new(app.router()).unwrap();
    let response = test.get("/test/tuple-config-test").await;

    assert_eq!(response.status_code(), 200);
    let json = response.json::<ResponseBody>();
    let data = json.data.unwrap();
    assert_eq!(data["config_type"], "tuple");
    assert_eq!(data["config"], json!(["jpg", "png"]));
}

#[tokio::test]
async fn array_config_middleware_test() {
    let app = Application::builder()
        .with_controller::<TestController>()
        .build();

    let test = TestServer::new(app.router()).unwrap();
    let response = test.get("/test/array-config-test").await;

    assert_eq!(response.status_code(), 200);
    let json = response.json::<ResponseBody>();
    let data = json.data.unwrap();
    assert_eq!(data["config_type"], "array");
    assert_eq!(data["config"], json!([1, 2, 3]));
}

#[tokio::test]
async fn string_config_middleware_test() {
    let app = Application::builder()
        .with_controller::<TestController>()
        .build();

    let test = TestServer::new(app.router()).unwrap();
    let response = test.get("/test/string-config-test").await;

    assert_eq!(response.status_code(), 200);
    let json = response.json::<ResponseBody>();
    let data = json.data.unwrap();
    assert_eq!(data["config_type"], "string");
    assert_eq!(data["config"], "test string");
}

#[tokio::test]
async fn str_config_middleware_test() {
    let app = Application::builder()
        .with_controller::<TestController>()
        .build();

    let test = TestServer::new(app.router()).unwrap();
    let response = test.get("/test/str-config-test").await;

    assert_eq!(response.status_code(), 200);
    let json = response.json::<ResponseBody>();
    let data = json.data.unwrap();
    assert_eq!(data["config_type"], "str");
    assert_eq!(data["config"], "test str");
}

#[tokio::test]
async fn number_config_middleware_test() {
    let app = Application::builder()
        .with_controller::<TestController>()
        .build();

    let test = TestServer::new(app.router()).unwrap();
    let response = test.get("/test/number-config-test").await;

    assert_eq!(response.status_code(), 200);
    let json = response.json::<ResponseBody>();
    let data = json.data.unwrap();
    assert_eq!(data["config_type"], "number");
    assert_eq!(data["config"], 42);
}

#[tokio::test]
async fn bool_config_middleware_test() {
    let app = Application::builder()
        .with_controller::<TestController>()
        .build();

    let test = TestServer::new(app.router()).unwrap();
    let response = test.get("/test/bool-config-test").await;

    assert_eq!(response.status_code(), 200);
    let json = response.json::<ResponseBody>();
    let data = json.data.unwrap();
    assert_eq!(data["config_type"], "bool");
    assert_eq!(data["config"], true);
}
