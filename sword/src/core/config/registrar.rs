use serde::de::DeserializeOwned;

use crate::core::{Config, ConfigError, State};

type RegisterConfigFn = fn(&Config, &State) -> Result<(), ConfigError>;

/// A struct that holds a function to register a config type.
/// Used by the inventory system to collect all config types at compile time.
pub struct ConfigRegistrar {
    pub register: RegisterConfigFn,
}

impl ConfigRegistrar {
    pub const fn new(register: RegisterConfigFn) -> Self {
        Self { register }
    }
}

inventory::collect!(ConfigRegistrar);

/// Trait for configuration section types.
///
/// Types implementing this trait can be used with `Config::get()` to extract
/// and deserialize specific sections from the configuration table.
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
    fn register(config: &Config, state: &State) -> Result<(), ConfigError> {
        Ok(state.insert(config.get::<Self>()?))
    }
}
