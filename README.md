# Sword

> <img src="https://avatars.githubusercontent.com/u/228345998?s=200&v=4" align="right" width="120"/>

Structured web framework for rust built on top of axum.  
Designed to build server application with less boilerplate and more simplicity.  
It takes advantage of the tokio and axum ecosystem to bring you performance with nice DX.

> Sword is in active development, expect breaking changes.

## Features

- **Macro-based routing** - Clean and intuitive route definitions
- **JSON-first design** - Built with JSON formats as priority
- **Built-in validation** - Support `validator` crate and extensible validation system
- **HTTP responses standarization** - Using `axum_responses` crate
- **Dependency Injection** - Built-in DI support with declarative macros
- **Asynchronous by default** - Built on top of `axum` and `tokio`
- **Interactive CLI** - Built to improve the developer experience

## Usage

### Add to your `Cargo.toml`

```toml
[dependencies]
sword = "0.2.1"
```

### Basic web server

```rust
use sword::prelude::*;
use serde_json::Value;

#[controller("/")]
struct AppController;

impl AppController {
    #[get("/hello")]
    async fn hello(&self) -> JsonResponse {
        JsonResponse::Ok().message("Hello, World!")
    }

    #[post("/submit")]
    async fn submit_data(&self, req: Request) -> HttpResult {
        let body = req.body::<Value>()?;

        Ok(JsonResponse::Ok()
            .data(body)
            .message("Data submitted successfully"))
    }
}

#[sword::main]
async fn main() {
    let app = Application::builder()
        .with_module::<AppController>()
        .build();

    app.run().await;
}
```

## More Examples

See the [examples directory](./examples) for more advanced usage.

## Changelog

See [CHANGELOG.md](./CHANGELOG.md) for more details.

## Contributing

Contributions are welcome! Please open an issue or submit a pull request. See [CONTRIBUTING.md](./CONTRIBUTING.md) for more details.
