use crate::{
    RwMap,
    core::{Build, Injectable},
};

use std::{any::TypeId, collections::HashMap, sync::Arc};

/// Marker trait for pre-instantiated dependencies (providers).
///
/// Providers are dependencies that cannot be auto-constructed from the State
/// (e.g., database connections, external API clients) but need to be available
/// for injection into other components.
pub trait Provider: Build {}

pub struct ProviderRegistry {
    providers: RwMap<TypeId, Injectable>,
}

impl ProviderRegistry {
    pub(crate) fn new() -> Self {
        Self {
            providers: RwMap::new(HashMap::new()),
        }
    }

    /// Registers a provider instance.
    ///
    /// Providers are instances that have already been constructed and are ready
    /// to be injected into other components. Typical use cases include database
    /// connections, HTTP clients, or external service configurations that cannot
    /// be auto-constructed from the State.
    pub fn register<T>(&self, provider: T)
    where
        T: Provider,
    {
        self.providers
            .write()
            .insert(TypeId::of::<T>(), Arc::new(provider));
    }

    pub(crate) fn get_providers(&self) -> &RwMap<TypeId, Injectable> {
        &self.providers
    }

    pub(crate) fn clear(&self) {
        self.providers.write().clear();
    }
}
