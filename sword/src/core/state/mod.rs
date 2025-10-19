mod error;
mod traits;

use std::{
    any::{Any, TypeId},
    collections::HashMap,
    sync::{Arc, RwLock},
};

pub use error::StateError;
pub use traits::*;

/// Application state container for type-safe dependency injection and data sharing.
///
/// `State` provides a thread-safe way to store and retrieve shared data across
/// the entire application. It uses `TypeId` as keys to ensure type safety and
/// prevents type confusion. State is automatically managed by the framework.
#[derive(Clone, Debug)]
pub struct State {
    inner: Arc<RwLock<HashMap<TypeId, Arc<dyn Any + Send + Sync>>>>,
}

impl State {
    pub(crate) fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Extract a clone of the stored value of type `T` from the state.
    pub fn get<T>(&self) -> Result<T, StateError>
    where
        T: Clone + Send + Sync + 'static,
    {
        let map = self.inner.read().map_err(|_| StateError::LockError)?;
        let type_name = std::any::type_name::<T>().to_string();

        let state_ref =
            map.get(&TypeId::of::<T>())
                .ok_or(StateError::TypeNotFound {
                    type_name: type_name.to_string(),
                })?;

        state_ref
            .downcast_ref::<T>()
            .cloned()
            .ok_or(StateError::TypeNotFound { type_name })
    }

    /// Borrow an `Arc` to the stored value of type `T` from the state.
    /// This returns an `Arc<T>` without cloning the underlying value.
    pub fn borrow<T>(&self) -> Result<Arc<T>, StateError>
    where
        T: Send + Sync + 'static,
    {
        let map = self.inner.read().map_err(|_| StateError::LockError)?;
        let type_name = std::any::type_name::<T>().to_string();

        let state_ref =
            map.get(&TypeId::of::<T>())
                .ok_or_else(|| StateError::TypeNotFound {
                    type_name: type_name.clone(),
                })?;

        state_ref
            .clone()
            .downcast::<T>()
            .map_err(|_| StateError::TypeNotFound { type_name })
    }

    pub fn insert<T: Send + Sync + 'static>(
        &self,
        state: T,
    ) -> Result<(), StateError> {
        self.inner
            .write()
            .map_err(|_| StateError::LockError)?
            .insert(TypeId::of::<T>(), Arc::new(state));

        Ok(())
    }

    pub(crate) fn insert_dependency(
        &self,
        type_id: TypeId,
        instance: Arc<dyn Any + Send + Sync>,
    ) -> Result<(), StateError> {
        self.inner
            .write()
            .map_err(|_| StateError::LockError)?
            .insert(type_id, instance);

        Ok(())
    }
}

impl Default for State {
    fn default() -> Self {
        Self::new()
    }
}
