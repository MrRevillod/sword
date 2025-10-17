# Middleware migration idea

### Overview

```rust
#[middleware]
pub struct AuthMiddleware {
    jwt_service: Arc<JwtService>,
}

impl AuthMiddleware {
    #[on_request]
    async fn some_name(
        &self,
        config: &'static str,
        ctx: Context,
        next: Next
    ) -> HttpResult {}
}
```

Por debajo, #[middleware] registra el builder del middleware en inventory para que al iniciar el servidor se pueda construir la cadena de middlewares.

```rust

#[controller]
pub struct MyController {}


impl MyController {
    #[get("/")]
    #[use_middleware(AuthMiddleware, config = "value")]
    async fn index(&self, ctx: Context) -> HttpResult {}
}
```
