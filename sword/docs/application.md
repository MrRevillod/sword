# ApplicationBuilder
Builder for constructing a Sword application with various configuration options.

`ApplicationBuilder` provides a fluent interface for configuring a Sword application
before building the final `Application` instance. It allows you to register
controllers, add middleware layers, configure shared state, and set up dependency injection.

### Example

```rust,ignore
use sword::prelude::*;

#[controller]
struct HomeController;

let app = Application::builder()
    .with_controller::<HomeController>()
    .with_layer(tower_http::cors::CorsLayer::permissive())
    .build();
```


## new

Creates a new application builder with default configuration.

This method initializes a new builder with:
- Empty router
- Fresh state container
- Configuration loaded from `config/config.toml`

### Returns

Returns `Ok(ApplicationBuilder)` if initialization succeeds, or
`Err(ApplicationError)` if configuration loading fails.

### Errors

This function will return an error if:
- The configuration file cannot be found or read
- The TOML syntax is invalid
- Environment variable interpolation fails

## with_layer

Registers a middleware layer in the application.

This method allows you to add Tower-based middleware or other layers
that implement the `Layer` trait. Layers are applied to all routes
in the application and can modify requests and responses.

### Arguments

* `layer` - The middleware layer to add to the application

### Example

```rust,ignore
use sword::prelude::*;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;

let app = Application::builder()
    .with_layer(CorsLayer::permissive())
    .with_layer(TraceLayer::new_for_http())
    .build();
```
## build

Builds the final application instance.

This method finalizes the application configuration and creates the
`Application` instance. It applies all configured middleware layers,
sets up request body limits, and prepares the application for running.

### Built-in Middleware

The following middleware is automatically applied:
- Content-Type validation middleware
- Request body size limiting middleware
- Cookie management layer (if `cookies` feature is enabled)