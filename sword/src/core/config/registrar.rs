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
