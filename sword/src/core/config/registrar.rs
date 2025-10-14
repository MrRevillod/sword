use crate::{
    core::{Config, State},
    errors::ConfigError,
};

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

// /// Registers all config types that were marked with `#[config]` into the application state.
// /// This is called automatically during application initialization.
// pub(crate) fn register_all_configs(
//     config: &Config,
//     state: &super::State,
// ) -> Result<(), ConfigError> {
//     for registrar in inventory::iter::<ConfigRegistrar> {
//         (registrar.register)(config, state)?;
//     }
//     Ok(())
// }

// Collect all ConfigRegistrar instances submitted via inventory::submit!
inventory::collect!(ConfigRegistrar);
