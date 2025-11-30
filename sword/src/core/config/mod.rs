mod env;
mod error;
mod registrar;

use env::expand_env_variables;
use serde::de::{DeserializeOwned, IntoDeserializer};
use std::{env::current_exe, fs, path::Path, str::FromStr, sync::Arc};
use toml::{Table, Value};

pub use error::ConfigError;
pub use registrar::*;
pub use sword_macros::config;

/// Struct representing the application's configuration.
///
/// This struct loads and holds the configuration data from a TOML file,
/// allowing retrieval of specific configuration sections through the `get` method.
#[derive(Debug, Clone, Default)]
pub struct Config {
    inner: Arc<Table>,
}

impl Config {
    pub(crate) fn new() -> Result<Self, ConfigError> {
        let content = Self::load_config_file()?;

        let expanded = expand_env_variables(&content)
            .map_err(ConfigError::InterpolationError)?;

        Ok(Self {
            inner: Arc::new(Table::from_str(&expanded)?),
        })
    }

    /// Retrieves and deserializes a configuration section.
    ///
    /// This method extracts a specific section from the loaded TOML configuration
    /// and deserializes it to the specified type.
    ///
    /// The `T` type must implement both
    /// `DeserializeOwned` for parsing and `ConfigItem` to specify which section
    /// to load from.
    ///
    /// The `ConfigItem` is implemented using the `#[config(key = "section_name")]` macro
    pub fn get<T: DeserializeOwned + ConfigItem>(&self) -> Result<T, ConfigError> {
        let key = T::toml_key();

        let Some(config_item) = self.inner.get(key).cloned() else {
            return Err(ConfigError::KeyNotFound(key.to_owned()));
        };

        let value = Value::into_deserializer(config_item);

        Ok(T::deserialize(value)?)
    }

    pub fn get_or_default<T: DeserializeOwned + ConfigItem + Default>(
        &self,
    ) -> Result<T, ConfigError> {
        match self.get::<T>() {
            Ok(value) => Ok(value),
            Err(ConfigError::KeyNotFound(_)) => Ok(T::default()),
            Err(e) => Err(e),
        }
    }

    pub fn get_or_panic<T: DeserializeOwned + ConfigItem>(&self) -> T {
        self.get::<T>().expect(&format!(
            "Failed to get configuration for key '{}'",
            T::toml_key()
        ))
    }

    fn load_config_file() -> Result<String, ConfigError> {
        let primary_path = Path::new("config/config.toml");

        if primary_path.exists() {
            return Ok(fs::read_to_string(primary_path)?);
        }

        Self::load_from_exe_directory()
    }

    fn load_from_exe_directory() -> Result<String, ConfigError> {
        let exe_path = current_exe().map_err(|_| ConfigError::FileNotFound)?;
        let exe_dir = exe_path.parent().ok_or(ConfigError::FileNotFound)?;

        let fallback_path = exe_dir.join("config/config.toml");

        if !fallback_path.exists() {
            return Err(ConfigError::FileNotFound);
        }

        Ok(fs::read_to_string(fallback_path)?)
    }
}
