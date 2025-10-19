// use axum_test::TestServer;
// use serde_json::json;
// use sword::prelude::*;
// use tower_http::cors::CorsLayer;

// #[derive(Debug, Clone)]
// pub enum LogLevel {
//     Debug,
//     Info,
//     Warning,
//     Error,
// }

// #[derive(Debug, Clone)]
// #[allow(unused)]
// pub enum DatabaseConfig {
//     Sqlite(String),
//     Postgres { host: String, port: u16 },
//     Memory,
// }

// #[derive(Debug, Clone)]
// #[allow(unused)]
// pub enum AuthMethod {
//     Basic { username: String, password: String },
//     None,
// }

// pub struct FileValidationMiddleware;

// impl MiddlewareWithConfig<(&str, &str)> for FileValidationMiddleware {
//     async fn handle(
//         config: (&str, &str),
//         mut req: Request,
//         next: Next,
//     ) -> MiddlewareResult {
//         req.extensions
//             .insert((config.0.to_string(), config.1.to_string()));

//         next!(req, next)
//     }
// }

// struct ExtensionsTestMiddleware;

// impl Middleware for ExtensionsTestMiddleware {
//     async fn handle(mut req: Request, next: Next) -> MiddlewareResult {
//         req.extensions
//             .insert::<String>("test_extension".to_string());

//         next!(req, next)
//     }
// }

// struct MwWithState;

// impl Middleware for MwWithState {
//     async fn handle(mut req: Request, next: Next) -> MiddlewareResult {
//         req.extensions.insert::<u16>(8080);
//         next!(req, next)
//     }
// }

// struct RoleMiddleware;

// impl MiddlewareWithConfig<Vec<&str>> for RoleMiddleware {
//     async fn handle(
//         roles: Vec<&str>,
//         mut req: Request,
//         next: Next,
//     ) -> MiddlewareResult {
//         let roles_owned: Vec<String> = roles.iter().map(|s| s.to_string()).collect();
//         req.extensions.insert(roles_owned);

//         next!(req, next)
//     }
// }

// struct TupleConfigMiddleware;

// impl MiddlewareWithConfig<(&str, &str)> for TupleConfigMiddleware {
//     async fn handle(
//         config: (&str, &str),
//         mut req: Request,
//         next: Next,
//     ) -> MiddlewareResult {
//         req.extensions
//             .insert((config.0.to_string(), config.1.to_string()));
//         next!(req, next)
//     }
// }

// struct ArrayConfigMiddleware;

// impl MiddlewareWithConfig<[i32; 3]> for ArrayConfigMiddleware {
//     async fn handle(
//         config: [i32; 3],
//         mut req: Request,
//         next: Next,
//     ) -> MiddlewareResult {
//         req.extensions.insert(config);

//         next!(req, next)
//     }
// }

// struct StringConfigMiddleware;

// impl MiddlewareWithConfig<String> for StringConfigMiddleware {
//     async fn handle(
//         config: String,
//         mut req: Request,
//         next: Next,
//     ) -> MiddlewareResult {
//         req.extensions.insert(config);

//         next!(req, next)
//     }
// }

// struct StrConfigMiddleware;

// impl MiddlewareWithConfig<&'static str> for StrConfigMiddleware {
//     async fn handle(
//         config: &'static str,
//         mut req: Request,
//         next: Next,
//     ) -> MiddlewareResult {
//         req.extensions.insert(config.to_string());

//         next!(req, next)
//     }
// }

// struct NumberConfigMiddleware;
// impl MiddlewareWithConfig<i32> for NumberConfigMiddleware {
//     async fn handle(config: i32, mut req: Request, next: Next) -> MiddlewareResult {
//         req.extensions.insert(config);

//         next!(req, next)
//     }
// }

// struct BoolConfigMiddleware;
// impl MiddlewareWithConfig<bool> for BoolConfigMiddleware {
//     async fn handle(config: bool, mut req: Request, next: Next) -> MiddlewareResult {
//         req.extensions.insert(config);

//         next!(req, next)
//     }
// }

// struct ComplexConfigMiddleware;

// impl MiddlewareWithConfig<(Vec<&str>, i32, bool)> for ComplexConfigMiddleware {
//     async fn handle(
//         config: (Vec<&str>, i32, bool),
//         mut req: Request,
//         next: Next,
//     ) -> MiddlewareResult {
//         let owned_config = (
//             config
//                 .0
//                 .iter()
//                 .map(|s| s.to_string())
//                 .collect::<Vec<String>>(),
//             config.1,
//             config.2,
//         );

//         req.extensions.insert(owned_config);

//         next!(req, next)
//     }
// }

// struct FunctionConfigMiddleware;
// impl MiddlewareWithConfig<Vec<String>> for FunctionConfigMiddleware {
//     async fn handle(
//         config: Vec<String>,
//         mut req: Request,
//         next: Next,
//     ) -> MiddlewareResult {
//         req.extensions.insert(config);

//         next!(req, next)
//     }
// }

// fn create_test_vector() -> Vec<String> {
//     vec!["test1".to_string(), "test2".to_string()]
// }

// struct MathConfigMiddleware;

// impl MiddlewareWithConfig<i32> for MathConfigMiddleware {
//     async fn handle(config: i32, mut req: Request, next: Next) -> MiddlewareResult {
//         req.extensions.insert(config);

//         next!(req, next)
//     }
// }

// struct ConstConfigMiddleware;

// impl MiddlewareWithConfig<&'static str> for ConstConfigMiddleware {
//     async fn handle(
//         config: &'static str,
//         mut req: Request,
//         next: Next,
//     ) -> MiddlewareResult {
//         req.extensions.insert(config.to_string());

//         next!(req, next)
//     }
// }

// const TEST_CONST: &str = "const_value";

// struct LogMiddleware;

// impl MiddlewareWithConfig<LogLevel> for LogMiddleware {
//     async fn handle(
//         config: LogLevel,
//         mut req: Request,
//         next: Next,
//     ) -> MiddlewareResult {
//         req.extensions.insert(config);

//         next!(req, next)
//     }
// }

// struct DatabaseMiddleware;

// impl MiddlewareWithConfig<DatabaseConfig> for DatabaseMiddleware {
//     async fn handle(
//         config: DatabaseConfig,
//         mut req: Request,
//         next: Next,
//     ) -> MiddlewareResult {
//         req.extensions.insert(config);

//         next!(req, next)
//     }
// }

// struct AuthMiddleware;

// impl MiddlewareWithConfig<AuthMethod> for AuthMiddleware {
//     async fn handle(
//         config: AuthMethod,
//         mut req: Request,
//         next: Next,
//     ) -> MiddlewareResult {
//         req.extensions.insert(config);

//         next!(req, next)
//     }
// }

// struct EnumOptionMiddleware;

// impl MiddlewareWithConfig<Option<LogLevel>> for EnumOptionMiddleware {
//     async fn handle(
//         config: Option<LogLevel>,
//         mut req: Request,
//         next: Next,
//     ) -> MiddlewareResult {
//         req.extensions.insert(config);

//         next!(req, next)
//     }
// }

// struct EnumVecMiddleware;

// impl MiddlewareWithConfig<Vec<LogLevel>> for EnumVecMiddleware {
//     async fn handle(
//         config: Vec<LogLevel>,
//         mut req: Request,
//         next: Next,
//     ) -> MiddlewareResult {
//         req.extensions.insert(config);

//         next!(req, next)
//     }
// }

// #[controller("/test")]
// struct TestController {}

// #[routes]
// impl TestController {
//     #[get("/extensions-test")]
//     #[middleware(ExtensionsTestMiddleware)]
//     async fn extensions_test(&self, req: Request) -> HttpResponse {
//         let extension_value = req.extensions.get::<String>();

//         HttpResponse::Ok()
//             .message("Test controller response with extensions")
//             .data(json!({
//                 "extension_value": extension_value.cloned().unwrap_or_default()
//             }))
//     }

//     #[get("/middleware-state")]
//     #[middleware(ExtensionsTestMiddleware)]
//     #[middleware(MwWithState)]
//     async fn middleware_state(&self, req: Request) -> HttpResult<HttpResponse> {
//         let port = req.extensions.get::<u16>().cloned().unwrap_or(0);
//         let message = req.extensions.get::<String>().cloned().unwrap_or_default();

//         let json = json!({
//             "port": port,
//             "message": message
//         });

//         Ok(HttpResponse::Ok()
//             .message("Test controller response with middleware state")
//             .data(json))
//     }

//     #[get("/role-test")]
//     #[middleware(RoleMiddleware, config = vec!["admin", "user"])]
//     async fn role_test(&self, req: Request) -> HttpResponse {
//         let config = ctx
//             .extensions
//             .get::<Vec<String>>()
//             .cloned()
//             .unwrap_or_default();

//         HttpResponse::Ok().data(json!({
//             "roles": config
//         }))
//     }

//     #[get("/error-test")]
//     #[middleware(FileValidationMiddleware, config = ("jpg", "png"))]
//     async fn error_test(&self, req: Request) -> HttpResponse {
//         let config = ctx
//             .extensions
//             .get::<(String, String)>()
//             .cloned()
//             .unwrap_or(("".to_string(), "".to_string()));

//         HttpResponse::Ok().data(json!({
//             "allowed_formats": [config.0, config.1]
//         }))
//     }

//     #[get("/tower-middleware-test")]
//     #[middleware(CorsLayer::permissive())]
//     async fn tower_middleware_test(&self) -> HttpResponse {
//         HttpResponse::Ok()
//             .message("Test with tower middleware")
//             .data(json!({"middleware": "cors"}))
//     }

//     #[get("/tuple-config-test")]
//     #[middleware(TupleConfigMiddleware, config = ("jpg", "png"))]
//     async fn tuple_config_test(&self, req: Request) -> HttpResponse {
//         let config = ctx
//             .extensions
//             .get::<(String, String)>()
//             .cloned()
//             .unwrap_or(("".to_string(), "".to_string()));

//         HttpResponse::Ok().message("Tuple config test").data(json!({
//             "config_type": "tuple",
//             "config": [config.0, config.1]
//         }))
//     }

//     #[get("/array-config-test")]
//     #[middleware(ArrayConfigMiddleware, config = [1, 2, 3])]
//     async fn array_config_test(&self, req: Request) -> HttpResponse {
//         let config = ctx
//             .extensions
//             .get::<[i32; 3]>()
//             .cloned()
//             .unwrap_or([0, 0, 0]);

//         HttpResponse::Ok().message("Array config test").data(json!({
//             "config_type": "array",
//             "config": config
//         }))
//     }

//     #[get("/string-config-test")]
//     #[middleware(StringConfigMiddleware, config = "test string".to_string())]
//     async fn string_config_test(&self, req: Request) -> HttpResponse {
//         let config = req.extensions.get::<String>().cloned().unwrap_or_default();

//         HttpResponse::Ok()
//             .message("String config test")
//             .data(json!({
//                 "config_type": "string",
//                 "config": config
//             }))
//     }

//     #[get("/str-config-test")]
//     #[middleware(StrConfigMiddleware, config = "test str")]
//     async fn str_config_test(&self, req: Request) -> HttpResponse {
//         let config = req.extensions.get::<String>().cloned().unwrap_or_default();

//         HttpResponse::Ok().message("Str config test").data(json!({
//             "config_type": "str",
//             "config": config
//         }))
//     }

//     #[get("/number-config-test")]
//     #[middleware(NumberConfigMiddleware, config = 42)]
//     async fn number_config_test(&self, req: Request) -> HttpResponse {
//         let config = req.extensions.get::<i32>().cloned().unwrap_or(0);

//         HttpResponse::Ok()
//             .message("Number config test")
//             .data(json!({
//                 "config_type": "number",
//                 "config": config
//             }))
//     }

//     #[get("/bool-config-test")]
//     #[middleware(BoolConfigMiddleware, config = true)]
//     async fn bool_config_test(&self, req: Request) -> HttpResponse {
//         let config = req.extensions.get::<bool>().cloned().unwrap_or(false);

//         HttpResponse::Ok().message("Bool config test").data(json!({
//             "config_type": "bool",
//             "config": config
//         }))
//     }

//     #[get("/complex-config-test")]
//     #[middleware(ComplexConfigMiddleware, config = (vec!["a", "b"], 100, false))]
//     async fn complex_config_test(&self, req: Request) -> HttpResponse {
//         let config = ctx
//             .extensions
//             .get::<(Vec<String>, i32, bool)>()
//             .cloned()
//             .unwrap_or((vec![], 0, false));

//         HttpResponse::Ok()
//             .message("Complex config test")
//             .data(json!({
//                 "config_type": "complex",
//                 "config": {
//                     "items": config.0,
//                     "number": config.1,
//                     "flag": config.2
//                 }
//             }))
//     }

//     #[get("/function-config-test")]
//     #[middleware(FunctionConfigMiddleware, config = create_test_vector())]
//     async fn function_config_test(&self, req: Request) -> HttpResponse {
//         let config = ctx
//             .extensions
//             .get::<Vec<String>>()
//             .cloned()
//             .unwrap_or_default();

//         HttpResponse::Ok()
//             .message("Function config test")
//             .data(json!({
//                 "config_type": "function",
//                 "config": config
//             }))
//     }

//     #[get("/macro-config-test")]
//     #[middleware(RoleMiddleware, config = vec!["admin", "user", "guest"])]
//     async fn macro_config_test(&self, req: Request) -> HttpResponse {
//         let config = ctx
//             .extensions
//             .get::<Vec<String>>()
//             .cloned()
//             .unwrap_or_default();

//         HttpResponse::Ok().message("Macro config test").data(json!({
//             "config_type": "macro",
//             "config": config
//         }))
//     }

//     #[get("/nested-config-test")]
//     #[middleware(FileValidationMiddleware, config = (
//         if true { "jpg" } else { "png" },
//         match 1 { 1 => "small", _ => "large" }
//     ))]
//     async fn nested_config_test(&self, req: Request) -> HttpResponse {
//         let config = ctx
//             .extensions
//             .get::<(String, String)>()
//             .cloned()
//             .unwrap_or(("".to_string(), "".to_string()));

//         HttpResponse::Ok()
//             .message("Nested config test")
//             .data(json!({
//                 "config_type": "nested",
//                 "config": [config.0, config.1]
//             }))
//     }

//     #[get("/math-config-test")]
//     #[middleware(MathConfigMiddleware, config = 2 + 3 * 4 - 1)]
//     async fn math_config_test(&self, req: Request) -> HttpResponse {
//         let config = req.extensions.get::<i32>().cloned().unwrap_or(0);

//         HttpResponse::Ok().message("Math config test").data(json!({
//             "config_type": "math",
//             "config": config
//         }))
//     }

//     #[get("/const-config-test")]
//     #[middleware(ConstConfigMiddleware, config = TEST_CONST)]
//     async fn const_config_test(&self, req: Request) -> HttpResponse {
//         let config = req.extensions.get::<String>().cloned().unwrap_or_default();

//         HttpResponse::Ok().message("Const config test").data(json!({
//             "config_type": "const",
//             "config": config
//         }))
//     }

//     #[get("/multiline-config-test")]
//     #[middleware(ComplexConfigMiddleware, config = (
//         vec![
//             "line1",
//             "line2",
//             "line3"
//         ],
//         {
//             let x = 10;
//             let y = 20;
//             x + y
//         },
//         true
//     ))]
//     async fn multiline_config_test(&self, req: Request) -> HttpResponse {
//         let config = ctx
//             .extensions
//             .get::<(Vec<String>, i32, bool)>()
//             .cloned()
//             .unwrap_or((vec![], 0, false));

//         HttpResponse::Ok()
//             .message("Multiline config test")
//             .data(json!({
//                 "config_type": "multiline",
//                 "config": {
//                     "items": config.0,
//                     "number": config.1,
//                     "flag": config.2
//                 }
//             }))
//     }

//     #[get("/closure-config-test")]
//     #[middleware(FunctionConfigMiddleware, config = {
//         let mut result = Vec::new();
//         for i in 1..=3 {
//             result.push(format!("item_{}", i));
//         }
//         result
//     })]
//     async fn closure_config_test(&self, req: Request) -> HttpResponse {
//         let config = ctx
//             .extensions
//             .get::<Vec<String>>()
//             .cloned()
//             .unwrap_or_default();

//         HttpResponse::Ok()
//             .message("Closure config test")
//             .data(json!({
//                 "config_type": "closure",
//                 "config": config
//             }))
//     }

//     #[get("/enum-simple-test")]
//     #[middleware(LogMiddleware, config = LogLevel::Info)]
//     async fn enum_simple_test(&self, req: Request) -> HttpResponse {
//         let config = req.extensions.get::<LogLevel>().cloned();

//         HttpResponse::Ok().message("Enum simple test").data(json!({
//             "config_type": "enum_simple",
//             "config": format!("{:?}", config.unwrap_or(LogLevel::Info))
//         }))
//     }

//     #[get("/enum-with-data-test")]
//     #[middleware(DatabaseMiddleware, config = DatabaseConfig::Sqlite("test.db".to_string()))]
//     async fn enum_with_data_test(&self, req: Request) -> HttpResponse {
//         let config = req.extensions.get::<DatabaseConfig>().cloned();

//         HttpResponse::Ok()
//             .message("Enum with data test")
//             .data(json!({
//                 "config_type": "enum_with_data",
//                 "config": format!("{:?}", config.unwrap_or(DatabaseConfig::Memory))
//             }))
//     }

//     #[get("/enum-struct-variant-test")]
//     #[middleware(DatabaseMiddleware, config = DatabaseConfig::Postgres {
//         host: "localhost".to_string(),
//         port: 5432
//     })]
//     async fn enum_struct_variant_test(&self, req: Request) -> HttpResponse {
//         let config = req.extensions.get::<DatabaseConfig>().cloned();

//         HttpResponse::Ok()
//             .message("Enum struct variant test")
//             .data(json!({
//                 "config_type": "enum_struct_variant",
//                 "config": format!("{:?}", config.unwrap_or(DatabaseConfig::Memory))
//             }))
//     }

//     #[get("/enum-unit-variant-test")]
//     #[middleware(DatabaseMiddleware, config = DatabaseConfig::Memory)]
//     async fn enum_unit_variant_test(&self, req: Request) -> HttpResponse {
//         let config = req.extensions.get::<DatabaseConfig>().cloned();

//         HttpResponse::Ok()
//             .message("Enum unit variant test")
//             .data(json!({
//                 "config_type": "enum_unit_variant",
//                 "config": format!("{:?}", config.unwrap_or(DatabaseConfig::Memory))
//             }))
//     }

//     #[get("/enum-nested-test")]
//     #[middleware(AuthMiddleware, config = AuthMethod::Basic {
//         username: "admin".to_string(),
//         password: "secret".to_string()
//     })]
//     async fn enum_nested_test(&self, req: Request) -> HttpResponse {
//         let config = req.extensions.get::<AuthMethod>().cloned();

//         HttpResponse::Ok().message("Enum nested test").data(json!({
//             "config_type": "enum_nested",
//             "config": format!("{:?}", config.unwrap_or(AuthMethod::None))
//         }))
//     }

//     #[get("/enum-option-some-test")]
//     #[middleware(EnumOptionMiddleware, config = Some(LogLevel::Error))]
//     async fn enum_option_some_test(&self, req: Request) -> HttpResponse {
//         let config = ctx
//             .extensions
//             .get::<Option<LogLevel>>()
//             .cloned()
//             .unwrap_or(None);

//         HttpResponse::Ok()
//             .message("Enum option some test")
//             .data(json!({
//                 "config_type": "enum_option_some",
//                 "config": config.map(|c| format!("{:?}", c))
//             }))
//     }

//     #[get("/enum-option-none-test")]
//     #[middleware(EnumOptionMiddleware, config = None::<LogLevel>)]
//     async fn enum_option_none_test(&self, req: Request) -> HttpResponse {
//         let config = ctx
//             .extensions
//             .get::<Option<LogLevel>>()
//             .cloned()
//             .unwrap_or(None);

//         HttpResponse::Ok()
//             .message("Enum option none test")
//             .data(json!({
//                 "config_type": "enum_option_none",
//                 "config": config.map(|c| format!("{:?}", c))
//             }))
//     }

//     #[get("/enum-vec-test")]
//     #[middleware(EnumVecMiddleware, config = vec![LogLevel::Debug, LogLevel::Info, LogLevel::Warning])]
//     async fn enum_vec_test(&self, req: Request) -> HttpResponse {
//         let config = ctx
//             .extensions
//             .get::<Vec<LogLevel>>()
//             .cloned()
//             .unwrap_or_default();

//         HttpResponse::Ok().message("Enum vec test").data(json!({
//             "config_type": "enum_vec",
//             "config": config.iter().map(|c| format!("{:?}", c)).collect::<Vec<_>>()
//         }))
//     }

//     #[get("/enum-match-test")]
//     #[middleware(LogMiddleware, config = match std::env::var("LOG_LEVEL") {
//         Ok(level) if level == "debug" => LogLevel::Debug,
//         Ok(level) if level == "error" => LogLevel::Error,
//         _ => LogLevel::Info,
//     })]
//     async fn enum_match_test(&self, req: Request) -> HttpResponse {
//         let config = req.extensions.get::<LogLevel>().cloned();

//         HttpResponse::Ok().message("Enum match test").data(json!({
//             "config_type": "enum_match",
//             "config": format!("{:?}", config.unwrap_or(LogLevel::Info))
//         }))
//     }
// }

// #[tokio::test]
// async fn extensions_mw_test() {
//     let app = Application::builder()
//         .with_controller::<TestController>()
//         .build();

//     let test = TestServer::new(app.router()).unwrap();
//     let response = test.get("/test/extensions-test").await;
//     assert_eq!(response.status_code(), 200);

//     let json = response.json::<ResponseBody>();
//     assert!(json.data.is_some());

//     let data = json.data.unwrap();

//     assert_eq!(data["extension_value"], "test_extension");
// }

// #[tokio::test]
// async fn middleware_state() {
//     let app = Application::builder()
//         .with_controller::<TestController>()
//         .build();

//     let test = TestServer::new(app.router()).unwrap();
//     let response = test.get("/test/middleware-state").await;

//     assert_eq!(response.status_code(), 200);

//     let json = response.json::<ResponseBody>();

//     assert!(json.data.is_some());

//     let data = json.data.unwrap();

//     assert_eq!(data["port"], 8080);
//     assert_eq!(data["message"], "test_extension");
// }

// #[tokio::test]
// async fn role_middleware_test() {
//     let app = Application::builder()
//         .with_controller::<TestController>()
//         .build();

//     let test = TestServer::new(app.router()).unwrap();
//     let response = test.get("/test/role-test").await;

//     assert_eq!(response.status_code(), 200);

//     let json = response.json::<ResponseBody>();
//     assert!(json.data.is_some());

//     let data = json.data.unwrap();
//     assert_eq!(data["roles"], json!(["admin", "user"]));
// }

// #[tokio::test]
// async fn tower_middleware_test() {
//     let app = Application::builder()
//         .with_controller::<TestController>()
//         .build();

//     let test = TestServer::new(app.router()).unwrap();
//     let response = test.get("/test/tower-middleware-test").await;

//     assert_eq!(response.status_code(), 200);

//     let json = response.json::<ResponseBody>();
//     assert!(json.data.is_some());

//     let data = json.data.unwrap();
//     assert_eq!(data["middleware"], "cors");
// }

// #[tokio::test]
// async fn tuple_config_middleware_test() {
//     let app = Application::builder()
//         .with_controller::<TestController>()
//         .build();

//     let test = TestServer::new(app.router()).unwrap();
//     let response = test.get("/test/tuple-config-test").await;

//     assert_eq!(response.status_code(), 200);
//     let json = response.json::<ResponseBody>();
//     let data = json.data.unwrap();
//     assert_eq!(data["config_type"], "tuple");
//     assert_eq!(data["config"], json!(["jpg", "png"]));
// }

// #[tokio::test]
// async fn array_config_middleware_test() {
//     let app = Application::builder()
//         .with_controller::<TestController>()
//         .build();

//     let test = TestServer::new(app.router()).unwrap();
//     let response = test.get("/test/array-config-test").await;

//     assert_eq!(response.status_code(), 200);
//     let json = response.json::<ResponseBody>();
//     let data = json.data.unwrap();
//     assert_eq!(data["config_type"], "array");
//     assert_eq!(data["config"], json!([1, 2, 3]));
// }

// #[tokio::test]
// async fn string_config_middleware_test() {
//     let app = Application::builder()
//         .with_controller::<TestController>()
//         .build();

//     let test = TestServer::new(app.router()).unwrap();
//     let response = test.get("/test/string-config-test").await;

//     assert_eq!(response.status_code(), 200);
//     let json = response.json::<ResponseBody>();
//     let data = json.data.unwrap();
//     assert_eq!(data["config_type"], "string");
//     assert_eq!(data["config"], "test string");
// }

// #[tokio::test]
// async fn str_config_middleware_test() {
//     let app = Application::builder()
//         .with_controller::<TestController>()
//         .build();

//     let test = TestServer::new(app.router()).unwrap();
//     let response = test.get("/test/str-config-test").await;

//     assert_eq!(response.status_code(), 200);
//     let json = response.json::<ResponseBody>();
//     assert_eq!(json.data.unwrap()["config_type"], "str");
// }

// #[tokio::test]
// async fn number_config_middleware_test() {
//     let app = Application::builder()
//         .with_controller::<TestController>()
//         .build();

//     let test = TestServer::new(app.router()).unwrap();
//     let response = test.get("/test/number-config-test").await;

//     assert_eq!(response.status_code(), 200);
//     let json = response.json::<ResponseBody>();
//     let data = json.data.unwrap();
//     assert_eq!(data["config_type"], "number");
//     assert_eq!(data["config"], 42);
// }

// #[tokio::test]
// async fn bool_config_middleware_test() {
//     let app = Application::builder()
//         .with_controller::<TestController>()
//         .build();

//     let test = TestServer::new(app.router()).unwrap();
//     let response = test.get("/test/bool-config-test").await;

//     assert_eq!(response.status_code(), 200);
//     let json = response.json::<ResponseBody>();
//     let data = json.data.unwrap();
//     assert_eq!(data["config_type"], "bool");
//     assert_eq!(data["config"], true);
// }

// #[tokio::test]
// async fn complex_config_middleware_test() {
//     let app = Application::builder()
//         .with_controller::<TestController>()
//         .build();

//     let test = TestServer::new(app.router()).unwrap();
//     let response = test.get("/test/complex-config-test").await;

//     assert_eq!(response.status_code(), 200);
//     let json = response.json::<ResponseBody>();
//     assert_eq!(json.data.unwrap()["config_type"], "complex");
// }

// #[tokio::test]
// async fn function_config_middleware_test() {
//     let app = Application::builder()
//         .with_controller::<TestController>()
//         .build();

//     let test = TestServer::new(app.router()).unwrap();
//     let response = test.get("/test/function-config-test").await;

//     assert_eq!(response.status_code(), 200);
//     let json = response.json::<ResponseBody>();
//     assert_eq!(json.data.unwrap()["config_type"], "function");
// }

// #[tokio::test]
// async fn macro_config_middleware_test() {
//     let app = Application::builder()
//         .with_controller::<TestController>()
//         .build();

//     let test = TestServer::new(app.router()).unwrap();
//     let response = test.get("/test/macro-config-test").await;

//     assert_eq!(response.status_code(), 200);
//     let json = response.json::<ResponseBody>();
//     assert_eq!(json.data.unwrap()["config_type"], "macro");
// }

// #[tokio::test]
// async fn nested_config_middleware_test() {
//     let app = Application::builder()
//         .with_controller::<TestController>()
//         .build();

//     let test = TestServer::new(app.router()).unwrap();
//     let response = test.get("/test/nested-config-test").await;

//     assert_eq!(response.status_code(), 200);
//     let json = response.json::<ResponseBody>();
//     assert_eq!(json.data.unwrap()["config_type"], "nested");
// }

// #[tokio::test]
// async fn math_config_middleware_test() {
//     let app = Application::builder()
//         .with_controller::<TestController>()
//         .build();

//     let test = TestServer::new(app.router()).unwrap();
//     let response = test.get("/test/math-config-test").await;

//     assert_eq!(response.status_code(), 200);
//     let json = response.json::<ResponseBody>();
//     assert_eq!(json.data.unwrap()["config_type"], "math");
// }

// #[tokio::test]
// async fn const_config_middleware_test() {
//     let app = Application::builder()
//         .with_controller::<TestController>()
//         .build();

//     let test = TestServer::new(app.router()).unwrap();
//     let response = test.get("/test/const-config-test").await;

//     assert_eq!(response.status_code(), 200);
//     let json = response.json::<ResponseBody>();
//     assert_eq!(json.data.unwrap()["config_type"], "const");
// }

// #[tokio::test]
// async fn multiline_config_middleware_test() {
//     let app = Application::builder()
//         .with_controller::<TestController>()
//         .build();

//     let test = TestServer::new(app.router()).unwrap();
//     let response = test.get("/test/multiline-config-test").await;

//     assert_eq!(response.status_code(), 200);
//     let json = response.json::<ResponseBody>();
//     assert_eq!(json.data.unwrap()["config_type"], "multiline");
// }

// #[tokio::test]
// async fn closure_config_middleware_test() {
//     let app = Application::builder()
//         .with_controller::<TestController>()
//         .build();

//     let test = TestServer::new(app.router()).unwrap();
//     let response = test.get("/test/closure-config-test").await;

//     assert_eq!(response.status_code(), 200);
//     let json = response.json::<ResponseBody>();
//     assert_eq!(json.data.unwrap()["config_type"], "closure");
// }

// #[tokio::test]
// async fn enum_simple_test() {
//     let app = Application::builder()
//         .with_controller::<TestController>()
//         .build();

//     let test = TestServer::new(app.router()).unwrap();
//     let response = test.get("/test/enum-simple-test").await;

//     assert_eq!(response.status_code(), 200);
//     let json = response.json::<ResponseBody>();
//     assert_eq!(json.data.unwrap()["config_type"], "enum_simple");
// }

// #[tokio::test]
// async fn enum_with_data_test() {
//     let app = Application::builder()
//         .with_controller::<TestController>()
//         .build();

//     let test = TestServer::new(app.router()).unwrap();
//     let response = test.get("/test/enum-with-data-test").await;

//     assert_eq!(response.status_code(), 200);
//     let json = response.json::<ResponseBody>();
//     assert_eq!(json.data.unwrap()["config_type"], "enum_with_data");
// }

// #[tokio::test]
// async fn enum_struct_variant_test() {
//     let app = Application::builder()
//         .with_controller::<TestController>()
//         .build();

//     let test = TestServer::new(app.router()).unwrap();
//     let response = test.get("/test/enum-struct-variant-test").await;

//     assert_eq!(response.status_code(), 200);
//     let json = response.json::<ResponseBody>();
//     assert_eq!(json.data.unwrap()["config_type"], "enum_struct_variant");
// }

// #[tokio::test]
// async fn enum_unit_variant_test() {
//     let app = Application::builder()
//         .with_controller::<TestController>()
//         .build();

//     let test = TestServer::new(app.router()).unwrap();
//     let response = test.get("/test/enum-unit-variant-test").await;

//     assert_eq!(response.status_code(), 200);
//     let json = response.json::<ResponseBody>();
//     assert_eq!(json.data.unwrap()["config_type"], "enum_unit_variant");
// }

// #[tokio::test]
// async fn enum_nested_test() {
//     let app = Application::builder()
//         .with_controller::<TestController>()
//         .build();

//     let test = TestServer::new(app.router()).unwrap();
//     let response = test.get("/test/enum-nested-test").await;

//     assert_eq!(response.status_code(), 200);
//     let json = response.json::<ResponseBody>();
//     assert_eq!(json.data.unwrap()["config_type"], "enum_nested");
// }

// #[tokio::test]
// async fn enum_option_some_test() {
//     let app = Application::builder()
//         .with_controller::<TestController>()
//         .build();

//     let test = TestServer::new(app.router()).unwrap();
//     let response = test.get("/test/enum-option-some-test").await;

//     assert_eq!(response.status_code(), 200);
//     let json = response.json::<ResponseBody>();
//     assert_eq!(json.data.unwrap()["config_type"], "enum_option_some");
// }

// #[tokio::test]
// async fn enum_option_none_test() {
//     let app = Application::builder()
//         .with_controller::<TestController>()
//         .build();

//     let test = TestServer::new(app.router()).unwrap();
//     let response = test.get("/test/enum-option-none-test").await;

//     assert_eq!(response.status_code(), 200);
//     let json = response.json::<ResponseBody>();
//     assert_eq!(json.data.unwrap()["config_type"], "enum_option_none");
// }

// #[tokio::test]
// async fn enum_vec_test() {
//     let app = Application::builder()
//         .with_controller::<TestController>()
//         .build();

//     let test = TestServer::new(app.router()).unwrap();
//     let response = test.get("/test/enum-vec-test").await;

//     assert_eq!(response.status_code(), 200);
//     let json = response.json::<ResponseBody>();
//     assert_eq!(json.data.unwrap()["config_type"], "enum_vec");
// }

// #[tokio::test]
// async fn enum_match_test() {
//     let app = Application::builder()
//         .with_controller::<TestController>()
//         .build();

//     let test = TestServer::new(app.router()).unwrap();
//     let response = test.get("/test/enum-match-test").await;

//     assert_eq!(response.status_code(), 200);
//     let json = response.json::<ResponseBody>();
//     assert_eq!(json.data.unwrap()["config_type"], "enum_match");
// }
