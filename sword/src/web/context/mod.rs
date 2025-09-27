pub mod extract;
pub mod request;

#[cfg(feature = "multipart")]
pub mod multipart;

#[cfg(feature = "cookies")]
pub mod cookies;

use axum::{
    body::Bytes,
    http::{Extensions, Method, Uri},
};

use serde::de::DeserializeOwned;
use std::collections::HashMap;
#[cfg(feature = "shaku-di")]
use std::sync::Arc;

#[cfg(feature = "shaku-di")]
use shaku::{HasComponent, Interface, Module};

use crate::{
    core::State,
    core::{Config, ConfigItem},
    errors::{ConfigError, StateError},
};

/// Context represents the incoming request context in the Sword framework.
///
/// `Context` is the primary interface for accessing request data in Sword applications.
/// It provides access to request parameters, body data, HTTP method, headers, URI,
/// and the application's shared state. This struct is automatically extracted from
/// incoming HTTP requests and passed to route handlers and middleware.
///
/// ### Key Features
///
/// - **Parameter Access**: URL and query parameters
/// - **Header Management**: Read and modify request headers
/// - **Body Handling**: Access to request body as bytes
/// - **State Access**: Retrieve shared application state
/// - **Configuration**: Access to application configuration
#[derive(Debug, Clone)]
pub struct Context {
    params: HashMap<String, String>,
    body_bytes: Bytes,
    method: Method,
    headers: HashMap<String, String>,
    uri: Uri,
    state: State,
    /// Axum extensions for additional request metadata.
    pub extensions: Extensions,
}

impl Context {
    /// Retrieves shared state of type `T` from the application state container.
    ///
    /// This method allows access to any state that was registered during application
    /// building using `ApplicationBuilder::with_state()`. The state is returned
    /// wrapped in an `Arc` for safe sharing across threads.
    ///
    /// ### Type Parameters
    ///
    /// * `T` - The type of state to retrieve (must implement `Send + Sync + 'static + Clone`)
    ///
    /// ### Returns
    ///
    /// Returns `Ok(T)` containing the state if found, or `Err(StateError)`
    /// if the state type was not registered.
    ///
    /// ### Errors
    ///
    /// This function will return a `StateError::TypeNotFound` if the requested
    /// state type was not registered in the application.
    pub fn get_state<T>(&self) -> Result<T, StateError>
    where
        T: Clone + Send + Sync + 'static + Clone,
    {
        let value = self
            .state
            .get::<T>()
            .map_err(|_| StateError::TypeNotFound)?;

        Ok(value)
    }

    /// Retrieves a dependency from a Shaku dependency injection module.
    ///
    /// This method provides access to services registered in Shaku modules
    /// that were added to the application using `ApplicationBuilder::with_shaku_di_module()`.
    ///
    /// Available only when the `shaku-di` feature is enabled.
    ///
    /// ### Type Parameters
    ///
    /// * `M` - The module type containing the dependency (must implement required Shaku traits)
    /// * `I` - The interface type being resolved (must be a `dyn Interface`)
    ///
    /// ### Returns
    ///
    /// Returns `Ok(Arc<I>)` containing the resolved service, or `Err(StateError)`
    /// if the module or service is not found.
    ///
    /// ### Errors
    ///
    /// This function will return a `StateError::TypeNotFound` if the requested
    /// module type was not registered in the application.
    ///
    /// ### Example
    /// To see usage, We recommend checking the full example in the sword framework repository.
    /// [Shaku dependency injection example](https://github.com/sword-framework/sword/tree/main/examples/src/dependency_injection)
    #[cfg(feature = "shaku-di")]
    pub fn di<M, I>(&self) -> Result<Arc<I>, StateError>
    where
        M: Module + HasComponent<I> + Send + Sync + 'static,
        I: Interface + ?Sized + 'static,
    {
        let module = self
            .state
            .borrow::<M>()
            .map_err(|_| StateError::TypeNotFound)?;

        let interface = module.resolve();

        Ok(interface)
    }

    /// Retrieves a configuration item of type `T` from the application configuration.
    ///
    /// This method provides access to configuration values loaded from TOML files.
    /// The configuration type must implement the `ConfigItem` trait, which can be
    /// done using the `#[config(key = "section_name")]` attribute macro.
    ///
    /// ### Type Parameters
    ///
    /// * `T` - The configuration type (must implement `DeserializeOwned + ConfigItem`)
    ///
    /// ### Returns
    ///
    /// Returns `Ok(T)` containing the parsed configuration, or `Err(ConfigError)`
    /// if the configuration cannot be loaded or parsed.
    ///
    /// ### Errors
    ///
    /// This function will return an error if:
    /// - The configuration section is not found in the TOML file
    /// - The configuration cannot be deserialized to type `T`
    /// - The configuration file cannot be accessed from the application state
    ///
    /// ### Example
    ///
    /// Configuration file (`config/config.toml`):
    /// ```toml,ignore
    /// [database]
    /// host = "localhost"
    /// port = 5432
    /// name = "myapp"
    ///
    /// [application]
    /// host = "0.0.0.0"
    /// port = 3000
    /// body_limit = "10MB"
    /// request_timeout_seconds = 30
    /// graceful_shutdown = true
    /// ```
    ///
    /// Rust code:
    /// ```rust,ignore
    /// use sword::prelude::*;
    /// use serde::Deserialize;
    ///
    /// #[derive(Deserialize)]
    /// #[config(key = "database")]
    /// struct DatabaseConfig {
    ///     host: String,
    ///     port: u16,
    ///     name: String,
    /// }
    ///
    /// ... asuming the rest of the controller ...
    ///
    /// #[get("/db-info")]
    /// async fn db_info(&self, ctx: Context) -> HttpResult<HttpResponse> {
    ///     let db_config = ctx.config::<DatabaseConfig>()?;
    ///
    ///     Ok(HttpResponse::Ok().data(format!(
    ///         "DB Host: {}, Port: {}, Name: {}",
    ///         db_config.host, db_config.port, db_config.name
    ///     )))
    /// }
    /// ```
    pub fn config<T: DeserializeOwned + ConfigItem>(
        &self,
    ) -> Result<T, ConfigError> {
        let config = self
            .state
            .get::<Config>()
            .map_err(|e| ConfigError::GetConfigError(e.to_string()))?;

        config.get::<T>()
    }
}
