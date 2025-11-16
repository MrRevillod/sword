mod error;
mod registrar;

use serde::de::{DeserializeOwned, IntoDeserializer};
use std::{env, fs, path::Path, str::FromStr, sync::Arc};
use toml::Table;

pub use error::ConfigError;
pub use registrar::*;
pub use sword_macros::config;

#[derive(Debug, Clone, Default)]
pub struct Config {
    inner: Arc<Table>,
}

impl Config {
    pub(crate) fn new() -> Result<Self, ConfigError> {
        let path = Path::new("config/config.toml");

        let content = if path.exists() {
            fs::read_to_string(path).map_err(ConfigError::ReadError)?
        } else {
            let exe_path =
                env::current_exe().map_err(|_| ConfigError::FileNotFound)?;
            let exe_dir = exe_path.parent().ok_or(ConfigError::FileNotFound)?;

            let fallback_path = exe_dir.join("config/config.toml");

            if !fallback_path.exists() {
                return Err(ConfigError::FileNotFound);
            }

            fs::read_to_string(fallback_path).map_err(ConfigError::ReadError)?
        };

        let expanded = crate::core::utils::expand_env_vars(&content)
            .map_err(ConfigError::InterpolationError)?;

        Ok(Self {
            inner: Arc::new(Table::from_str(&expanded)?),
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
    pub fn get<T: DeserializeOwned + ConfigItem>(&self) -> Result<T, ConfigError> {
        let key = T::toml_key();

        let Some(config_item) = self.inner.get(key).cloned() else {
            return Err(ConfigError::KeyNotFound(key.to_owned()));
        };

        let value = toml::Value::into_deserializer(config_item);

        Ok(T::deserialize(value)?)
    }
}
