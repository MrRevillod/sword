<div align="center">
<img src="https://pillan.inf.uct.cl/~lrevillod/images/sword-logo.webp" alt="Sword Logo" width="200">

<h1>⚔️ Sword ⚔️</h1>
<p><em>A prototype for a rust web framework</em></p>
</div>

## ✨ Features

- 🛣️ **Macro-based routing** - Clean and intuitive route definitions
- 🔍 **Complex query parameters** - Ready for advanced parameter handling
- 📄 **JSON-first design** - Built with JSON formats as priority
- ✅ **Built-in validation** - Support with `serde` and `validator` crates
- 🌐 **RFC-compliant HTTP responses** - Using `axum_responses` crate
- 💡 **Express-Like** - It provides a `Context` object with utility methods for request handling
- 💉 **Dependency Injection** - Built-in DI support using `shaku` crate

## 🛠️ Usage

### Add to your `Cargo.toml`

```toml
[dependencies]
sword = "0.1.3"

# Additional dependencies for features

# validation features:
validator = { version = "0.20.0", features = ["derive"] }

# dependency injection features:
shaku = { version = "0.6.2", features = ["derive"] }
```

### Basic web server 

```rust
use sword::prelude::*;
use sword::http::Result;

#[controller("/")]
struct AppController {}

#[controller_impl]
impl AppController {
    #[get("/")]
    async fn get_data() -> HttpResponse {
        let data = vec![
            "This is a basic web server",
            "It serves static data",
            "You can extend it with more routes",
        ];

        HttpResponse::Ok().data(data)
    }

    #[get("/hello")]
    async fn hello() -> HttpResponse {
        HttpResponse::Ok().data("Hello, World!")
    }

    #[post("/submit")]
    async fn submit_data(ctx: Context) -> Result<HttpResponse> {
        let body = ctx.body::<serde_json::Value>()?;

        Ok(HttpResponse::Ok()
            .data(body)
            .message("Data submitted successfully"))
    }
}

#[tokio::main]
async fn main() {
    Application::builder()
        .controller::<AppController>()
        .run("0.0.0.0:8080")
        .await;
}
```

### With State Management

```rust
use serde_json::json;
use std::sync::{Arc, OnceLock};
use sword::prelude::*;
use sword::http::Result;
use tokio::sync::RwLock;

type InMemoryDb = Arc<RwLock<Vec<String>>>;
const IN_MEMORY_DB: OnceLock<InMemoryDb> = OnceLock::new();

fn db() -> Arc<RwLock<Vec<String>>> {
    IN_MEMORY_DB
        .get_or_init(|| Arc::new(RwLock::new(Vec::new())))
        .clone()
}

#[derive(Clone)]
struct AppState {
    db: InMemoryDb,
}

#[controller("/api")]
struct AppController {}

#[controller_impl]
impl AppController {
    #[get("/data")]
    async fn get_data(ctx: Context) -> Result<HttpResponse> {
        let state = ctx.get_state::<AppState>()?;
        let count = state.db.read().await.len();
        let message = format!("Current data count: {}", count);

        state.db.write().await.push(message);

        Ok(HttpResponse::Ok().data(json!({
            "count": count,
            "current_data": state.db.read().await.clone(),
        })))
    }
}

#[tokio::main]
async fn main() {
    let app_state = AppState { db: db() };

    // !Important: Set Application state before registering any controllers
    // This allows controllers to access the shared state.

    Application::builder()
        .state(app_state)
        .controller::<AppController>()
        .run("0.0.0.0:8080")
        .await;
}
```

### With Middleware

```rust
use serde_json::json;
use sword::prelude::*;
use sword::http::Result;

struct LoggingMiddleware;

impl Middleware for LoggingMiddleware {
    async fn handle(mut ctx: Context, next: Next) -> MiddlewareResult {
        println!("Request: {} {}", ctx.method(), ctx.uri());
        
        ctx.extensions.insert::<String>("middleware_data".to_string());
        Ok(next.run(ctx.into()).await)
    }
}

#[controller("/api")]
struct AppController {}

#[controller_impl]
impl AppController {
    #[get("/hello")]
    #[middleware(LoggingMiddleware)]
    async fn hello(ctx: Context) -> Result<HttpResponse> {
        let middleware_data = ctx.extensions
            .get::<String>()
            .cloned()
            .unwrap_or_default();

        Ok(HttpResponse::Ok().data(json!({
            "message": "Hello from middleware!",
            "middleware_data": middleware_data
        })))
    }
}

#[tokio::main]
async fn main() {
    Application::builder()
        .controller::<AppController>()
        .run("0.0.0.0:8080")
        .await;
}
```

### With Dependency Injection

```rust
use std::sync::Arc;
use serde_json::json;
use shaku::{module, Component, Interface};
use sword::prelude::*;
use sword::http::Result;

trait Logger: Interface {
    fn log(&self, message: &str);
}

#[derive(Component)]
#[shaku(interface = Logger)]
struct ConsoleLogger;

impl Logger for ConsoleLogger {
    fn log(&self, message: &str) {
        println!("Log: {}", message);
    }
}

module! {
    AppModule {
        components = [ConsoleLogger],
        providers = []
    }
}

#[controller("/users")]
struct UserController {}

#[controller_impl]
impl UserController {
    #[get("/")]
    async fn get_users(ctx: Context) -> Result<HttpResponse> {
        let logger = ctx.get_dependency::<AppModule, dyn Logger>()?;
        logger.log("Fetching users");

        Ok(HttpResponse::Ok()
            .data(json!({
                "users": ["Alice", "Bob", "Charlie"]
            }))
            .message("Users retrieved successfully"))
    }
}

#[tokio::main]
async fn main() {
    let module = AppModule::builder().build();

    Application::builder()
        .di_module(module)
        .controller::<UserController>()
        .run("0.0.0.0:8080")
        .await;
}
```

### With Data Validation

```rust
use serde::{Deserialize, Serialize};
use sword::prelude::*;
use sword::http::Result;
use validator::Validate;

#[derive(Serialize, Deserialize, Validate)]
struct UserQuery {
    #[validate(range(message = "Page must be between 1 and 1000", min = 1, max = 1000))]
    page: u32,
    #[validate(range(message = "Limit must be between 1 and 100", min = 1, max = 100))]
    limit: u32,
}

#[derive(Serialize, Deserialize, Validate)]
struct CreateUserRequest {
    #[validate(length(min = 1, message = "Name must not be empty"))]
    name: String,
    #[validate(email(message = "Invalid email format"))]
    email: String,
}

#[controller("/users")]
struct UserController {}

#[controller_impl]
impl UserController {
    #[get("/")]
    async fn get_users(ctx: Context) -> Result<HttpResponse> {
        let query = ctx.validated_query::<UserQuery>()?;
        
        Ok(HttpResponse::Ok()
            .data(format!("Page: {}, Limit: {}", query.page, query.limit))
            .message("Users retrieved successfully"))
    }

    #[post("/")]
    async fn create_user(ctx: Context) -> Result<HttpResponse> {
        let user = ctx.validated_body::<CreateUserRequest>()?;

        Ok(HttpResponse::Ok()
            .data(user)
            .message("User created successfully"))
    }
}

#[tokio::main]
async fn main() {
    Application::builder()
        .controller::<UserController>()
        .run("0.0.0.0:8080")
        .await;
}
```

### With Middleware Configuration

```rust
use serde_json::json;
use sword::prelude::*;
use sword::http::Result;

struct RoleMiddleware;

impl MiddlewareWithConfig<Vec<&str>> for RoleMiddleware {
    async fn handle(roles: Vec<&str>, mut ctx: Context, next: Next) -> MiddlewareResult {
        println!("Required roles: {:?}", roles);
        ctx.extensions.insert(roles);

        Ok(next.run(ctx.into()).await)
    }
}

struct AuthenticationMiddleware;

impl MiddlewareWithConfig<String> for AuthenticationMiddleware {
    async fn handle(secret: String, mut ctx: Context, next: Next) -> MiddlewareResult {
        let auth_header = ctx.header("Authorization").unwrap_or("");
        
        if auth_header.is_empty() {
            return Ok(HttpResponse::Unauthorized()
                .message("Authorization header required")
                .into());
        }

        ctx.extensions.insert(secret);
        
        Ok(next.run(ctx.into()).await)
    }
}

#[controller("/admin")]
struct AdminController {}

#[controller_impl]
impl AdminController {
    #[get("/users")]
    #[middleware(RoleMiddleware, config = vec!["admin", "user"])]
    async fn get_users(ctx: Context) -> Result<HttpResponse> {
        let roles = ctx.extensions
            .get::<Vec<&str>>()
            .cloned()
            .unwrap_or_default();

        Ok(HttpResponse::Ok()
            .data(json!({
                "users": ["Alice", "Bob", "Charlie"],
                "required_roles": roles
            }))
            .message("Users retrieved successfully"))
    }

    #[post("/protected")]
    #[middleware(AuthenticationMiddleware, config = "super-secret-key".to_string())]
    async fn protected_endpoint(ctx: Context) -> Result<HttpResponse> {
        let secret = ctx.extensions
            .get::<String>()
            .cloned()
            .unwrap_or_default();

        Ok(HttpResponse::Ok()
            .data(json!({
                "message": "Access granted to protected resource",
                "secret_used": !secret.is_empty()
            })))
    }
}

#[tokio::main]
async fn main() {
    Application::builder()
        .controller::<AdminController>()
        .run("0.0.0.0:8080")
        .await;
}
```

### With Middleware Stacking

```rust
use serde_json::json;
use sword::prelude::*;
use sword::http::Result;

struct LoggingMiddleware;

impl Middleware for LoggingMiddleware {
    async fn handle(mut ctx: Context, next: Next) -> MiddlewareResult {
        println!("🔍 [LOGGING] Request: {} {}", ctx.method(), ctx.uri());
        ctx.extensions.insert::<String>("logged".to_string());
        
        let result = next.run(ctx.into()).await;
        println!("🔍 [LOGGING] Response completed");
        Ok(result)
    }
}

struct AuthMiddleware;

impl Middleware for AuthMiddleware {
    async fn handle(mut ctx: Context, next: Next) -> MiddlewareResult {
        println!("🔐 [AUTH] Checking authentication...");
        
        let auth_header = ctx.header("Authorization").unwrap_or("");
        if auth_header.is_empty() {
            println!("🔐 [AUTH] No authorization header found");
            return Ok(HttpResponse::Unauthorized()
                .message("Authorization required")
                .into());
        }
        
        ctx.extensions.insert::<String>("authenticated".to_string());
        println!("🔐 [AUTH] Authentication successful");
        Ok(next.run(ctx.into()).await)
    }
}

struct RoleMiddleware;

impl MiddlewareWithConfig<&str> for RoleMiddleware {
    async fn handle(required_role: &str, mut ctx: Context, next: Next) -> MiddlewareResult {
        let user_role = ctx.header("X-User-Role").unwrap_or("guest");
        
        if user_role != required_role && required_role != "guest" {
            return Ok(HttpResponse::Forbidden()
                .message("Insufficient permissions")
                .into());
        }
        
        ctx.extensions.insert::<String>(format!("role:{}", user_role));

        Ok(next.run(ctx.into()).await)
    }
}

struct TimingMiddleware;

impl Middleware for TimingMiddleware {
    async fn handle(mut ctx: Context, next: Next) -> MiddlewareResult {
        let start = std::time::Instant::now();
        println!("⏱️  [TIMING] Request started");
        
        ctx.extensions.insert::<std::time::Instant>(start);
        
        let result = next.run(ctx.into()).await;
        let duration = start.elapsed();
        
        println!("⏱️  [TIMING] Request completed in {:?}", duration);
        Ok(result)
    }
}

#[controller("/api")]
struct StackedController {}

#[controller_impl]
impl StackedController {
    #[get("/protected")]
    #[middleware(LoggingMiddleware)]           // Executes 1st (outer)
    #[middleware(TimingMiddleware)]            // Executes 2nd  
    #[middleware(AuthMiddleware)]              // Executes 3rd
    #[middleware(RoleMiddleware, config = "admin")] // Executes 4th (inner)
    async fn protected_endpoint(ctx: Context) -> Result<HttpResponse> {
        let logged = ctx.extensions.get::<String>().cloned().unwrap_or_default();
        let role_info = ctx.extensions.get::<String>().cloned().unwrap_or_default();
        
        Ok(HttpResponse::Ok()
            .data(json!({
                "middleware_chain": [
                    "LoggingMiddleware",
                    "TimingMiddleware", 
                    "AuthMiddleware",
                    "RoleMiddleware"
                ],
                "logged": !logged.is_empty(),
                "role": role_info
            }))
            .message("Protected endpoint accessed successfully"))
    }

    #[get("/public")]
    #[middleware(LoggingMiddleware)]
    #[middleware(TimingMiddleware)]
    async fn public_endpoint(ctx: Context) -> Result<HttpResponse> {
        Ok(HttpResponse::Ok()
            .data(json!({
                "message": "Public resource accessed",
                "middleware_chain": ["LoggingMiddleware", "TimingMiddleware"]
            })))
    }
}

#[tokio::main]
async fn main() {
    Application::builder()
        .controller::<StackedController>()
        .run("0.0.0.0:8080")
        .await;
}
```

## Currently working on
- ✅📱 Add Application struct
- ✅ 🏗️ Add Application Context
- ✅ 🔒 Add Middleware support
- ✅ 💉 Add Dependency Injection support based on `shaku` crate
- [ ] ⚙️ Add config file support

## 📋 Roadmap

- [ ] 📁 Add File - FormData support
- [ ] 🧪 Add more tests
- [ ] 📚 Add more documentation
- [ ] 🛠️ CLI Command line interface for code-generation (templates)


