# Sword

> <img src="https://avatars.githubusercontent.com/u/228345998?s=200&v=4" align="right" width="120"/>

Structured web framework for rust built on top tokio ecosystem.
Designed to build server application with less boilerplate and more simplicity.  
It takes advantage of the tokio ecosystem to bring you performance with nice DX.

> Sword is in active development, expect breaking changes.

## Features

- **Web Adapters** - Support for REST (Axum) and SocketIO (Socketioxide) adapters
- **Macro-based routing** - Clean and intuitive route definitions
- **JSON-first design** - Built with JSON formats as priority
- **Built-in validation** - Support `validator` crate and extensible validation system
- **HTTP responses standarization** - Consistent response formats out of the box
- **Dependency Injection** - Built-in DI support with declarative macros
- **Middleware and Interceptors** - Easy to use middleware and interceptor system
- TOML Config loader - Built-in support for loading configuration from TOML files

## Coming Soon

- **GraphQL Adapter** - Support for GraphQL APIs
- **gRPC Adapter** - Support for gRPC services with Tonic
- **RabbitMQ Integration** - Built-in support for RabbitMQ messaging

## Web Adapters Examples

```rust
use sword::prelude::*;

#[controller("/users")]
pub struct UsersController {
    hasher: Arc<Hasher>,
    users: Arc<UserRepository>,
}

impl UsersController {
    #[get("/")]
    async fn get_users(&self, req: Request) -> HttpResult {
        let data = self.users.find_all().await?;

        Ok(JsonResponse::Ok().data(data).request_id(req.id()))
    }

    #[post("/")]
    async fn create_user(&self, req: Request) -> HttpResult {
        let body = req.body_validator::<CreateUserDto>()?;
        let user = User::new(body.username, self.hasher.hash(&body.password)?);

        if self.users.find_by_username(&user.username).await?.is_some() {
            tracing::error!(
                "Attempt to create user with existing username: {}",
                user.username
            );

            return Err(AppError::UserConflictError("username", &user.username))?;
        }

        self.users.save(&user).await?;

        Ok(JsonResponse::Created().message("User created").data(user))
    }
}
```

## Full Examples

- [Rest API](./examples/http-controllers-adapter)
- [SocketIO Adapter Chat](./examples/socketio-adapter)
- [Interceptors (Both adapters)](./examples/interceptors)

## Changelog

See [CHANGELOG.md](./CHANGELOG.md) for more details.

## Contributing

Contributions are welcome! Please open an issue or submit a pull request. See [CONTRIBUTING.md](./CONTRIBUTING.md) for more details.
