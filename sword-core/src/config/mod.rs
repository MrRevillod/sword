mod env;
mod error;
mod registrar;
mod utils;

use env::expand_env_variables;
use serde::de::{DeserializeOwned, IntoDeserializer};
use std::{env::current_exe, fs, path::Path, str::FromStr, sync::Arc};
use toml::{Table, Value};

pub use error::ConfigError;
pub use registrar::*;
pub use sword_macros::config;
pub use utils::{ByteConfig, TimeConfig};

const DEFAULT_CONFIG_PATH: &str = "config/config.toml";
const CONFIG_ENV_VAR: &str = "SWORD_CONFIG_PATH";

/// Application configuration loaded from TOML files.
///
/// Loads configuration with the following priority:
/// 1. Explicit path via `from_path()`
/// 2. `SWORD_CONFIG_PATH` environment variable
/// 3. `config/config.toml`
/// 4. `<executable_dir>/config/config.toml`
#[derive(Debug, Clone, Default)]
pub struct Config {
    inner: Arc<Table>,
}

impl Config {
    /// Loads configuration using default priority order.
    ///
    /// # Errors
    ///
    /// Returns `ConfigError` if no configuration file is found or contains invalid TOML.
    pub fn new() -> Result<Self, ConfigError> {
        let content = Self::load_config_file(None)?;

        let expanded = expand_env_variables(&content)
            .map_err(ConfigError::interpolation_error)?;

        Ok(Self {
            inner: Arc::new(Table::from_str(&expanded)?),
        })
    }

    /// Loads configuration from a specific file path without fallbacks.
    ///
    /// # Errors
    ///
    /// Returns `ConfigError::FileNotFound` if the file doesn't exist.
    pub fn from_path<P: AsRef<Path>>(path: P) -> Result<Self, ConfigError> {
        let content = Self::load_config_file(Some(path.as_ref()))?;

        let expanded = expand_env_variables(&content)
            .map_err(ConfigError::interpolation_error)?;

        Ok(Self {
            inner: Arc::new(Table::from_str(&expanded)?),
        })
    }

    /// Retrieves and deserializes a configuration section.
    ///
    /// Type `T` must implement `ConfigItem` via `#[config(key = "section")]` macro.
    pub fn get<T: DeserializeOwned + ConfigItem>(&self) -> Result<T, ConfigError> {
        let key = T::toml_key();

        let Some(config_item) = self.inner.get(key).cloned() else {
            return Err(ConfigError::key_not_found(key));
        };

        let value = Value::into_deserializer(config_item);

        Ok(T::deserialize(value)?)
    }

    /// Retrieves a configuration section, panicking if not found or invalid.
    pub fn get_or_panic<T: DeserializeOwned + ConfigItem>(&self) -> T {
        self.get::<T>().unwrap_or_else(|_| {
            panic!("Failed to load configuration for key '{}'", T::toml_key())
        })
    }

    /// Retrieves a configuration section, returning default if not found or invalid.
    pub fn get_or_default<T: DeserializeOwned + ConfigItem + Default>(&self) -> T {
        self.get::<T>().unwrap_or_default()
    }

    fn load_config_file(path: Option<&Path>) -> Result<String, ConfigError> {
        if let Some(p) = path {
            if p.exists() {
                return Ok(fs::read_to_string(p)?);
            }
            return Err(ConfigError::FileNotFound);
        }

        if let Ok(env_path) = std::env::var(CONFIG_ENV_VAR) {
            let env_path = Path::new(&env_path);

            if env_path.exists() {
                return Ok(fs::read_to_string(env_path)?);
            }

            eprintln!(
                "Warning: {} is set to '{}' but file does not exist. Falling back to default paths.",
                CONFIG_ENV_VAR,
                env_path.display()
            );
        }

        let default_path = Path::new(DEFAULT_CONFIG_PATH);

        if default_path.exists() {
            return Ok(fs::read_to_string(default_path)?);
        }

        Self::load_from_exe_directory()
    }

    fn load_from_exe_directory() -> Result<String, ConfigError> {
        let exe_path = current_exe().map_err(|_| ConfigError::FileNotFound)?;
        let exe_dir = exe_path.parent().ok_or(ConfigError::FileNotFound)?;

        let fallback_path = exe_dir.join(DEFAULT_CONFIG_PATH);

        if !fallback_path.exists() {
            return Err(ConfigError::FileNotFound);
        }

        Ok(fs::read_to_string(fallback_path)?)
    }
}
