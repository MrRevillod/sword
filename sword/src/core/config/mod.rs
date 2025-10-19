mod error;
mod registrar;

use serde::de::{DeserializeOwned, IntoDeserializer};
use std::{fs, path::Path, str::FromStr, sync::Arc};
use toml::Table;

pub use error::ConfigError;
pub use registrar::*;
pub use sword_macros::config;

use crate::core::State;

#[derive(Debug, Clone, Default)]
pub struct Config {
    inner: Arc<Table>,
}

/// Trait for configuration section types.
///
/// Types implementing this trait can be used with `Config::get()` to extract
/// and deserialize specific sections from the configuration file.
///
/// Use the `#[config(key = "section_name")]` macro to automatically implement this trait.
/// The macro will also auto-register the config type using the `inventory` crate.
///
/// ```rust,ignore
/// use sword::prelude::*;
///
/// #[config(key = "my_section")]
/// struct MyConfig {
///     value: String,
/// }
/// ```
pub trait ConfigItem: DeserializeOwned + Clone + Send + Sync + 'static {
    /// Returns the TOML section key for this configuration type.
    fn toml_key() -> &'static str;

    /// Registers this config type in the application State.
    /// This is called automatically during application bootstrap.
    fn register_in_state(config: &Config, state: &State) -> Result<(), ConfigError> {
        state.insert(config.get::<Self>()?).map_err(|_| {
            ConfigError::ParseError(format!(
                "Failed to register config '{}' in state",
                Self::toml_key()
            ))
        })
    }
}

impl Config {
    pub(crate) fn new() -> Result<Self, ConfigError> {
        let path = Path::new("config/config.toml");

        let content = if path.exists() {
            fs::read_to_string(path).map_err(ConfigError::ReadError)?
        } else {
            let exe_path = std::env::current_exe()
                .map_err(|_| ConfigError::FileNotFound("config/config.toml"))?;

            let exe_dir = exe_path
                .parent()
                .ok_or(ConfigError::FileNotFound("config/config.toml"))?;

            let fallback_path = exe_dir.join("config/config.toml");

            if !fallback_path.exists() {
                return Err(ConfigError::FileNotFound("config/config.toml"));
            }

            fs::read_to_string(fallback_path).map_err(ConfigError::ReadError)?
        };

        let expanded = crate::core::utils::expand_env_vars(&content)
            .map_err(ConfigError::InterpolationError)?;

        let table = Table::from_str(&expanded)
            .map_err(|e| ConfigError::ParseError(e.to_string()))?;

        Ok(Self {
            inner: Arc::new(table),
        })
    }

    /// Retrieves and deserializes a configuration section.
    ///
    /// This method extracts a specific section from the loaded TOML configuration
    /// and deserializes it to the specified type. The type must implement both
    /// `DeserializeOwned` for parsing and `ConfigItem` to specify which section
    /// to load from.
    ///
    /// ### Type Parameters
    ///
    /// * `T` - The configuration type to deserialize (must implement `DeserializeOwned`)
    ///
    /// ### Example
    ///
    /// ```rust,ignore
    /// use sword::prelude::*;
    /// use serde::Deserialize;
    ///
    /// #[derive(Deserialize)]
    /// #[config(key = "application")]
    /// struct DatabaseConfig {
    ///     url: String,
    /// }
    ///
    /// // Then in a route handler:
    ///
    /// #[get("/db-info")]
    /// async fn db_info(&self, ctx: Context) -> HttpResult<HttpResponse> {
    ///     let db_config = ctx.config::<DatabaseConfig>()?;
    ///     Ok(HttpResponse::Ok().data(db_config))
    /// }
    ///
    /// ```
    pub fn get<T: DeserializeOwned + ConfigItem>(&self) -> Result<T, ConfigError> {
        let Some(config_item) = self.inner.get(T::toml_key()) else {
            return Err(ConfigError::KeyNotFound(T::toml_key().to_string()));
        };

        let value = toml::Value::into_deserializer(config_item.clone());

        T::deserialize(value)
            .map_err(|e| ConfigError::DeserializeError(e.to_string()))
    }
}
