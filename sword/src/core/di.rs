use std::{
    any::{Any, TypeId},
    collections::{HashMap, HashSet},
    sync::Arc,
};

use crate::{core::State, errors::DependencyInjectionError};

type Dependency = Arc<dyn Any + Send + Sync>;

type DependencyBuilder =
    Box<dyn Fn(&State) -> Result<Dependency, DependencyInjectionError>>;

pub trait Injectable {
    fn build(state: &State) -> Result<Self, DependencyInjectionError>
    where
        Self: Sized;

    fn dependencies() -> Vec<TypeId>;
}

/// Marker trait for types that are manually instantiated and registered as providers.
///
/// Providers are dependencies that cannot be auto-constructed from the State
/// (e.g., database connections, external API clients) but need to be available
/// for injection into other services.
pub trait Provider: Send + Sync + 'static {}

/// A container for managing dependencies and their builders.
///
/// Basically it support two types of registrations:
///
/// 1. Intances:
///
/// Instances are pre-created objects that you want to register directly into the container.
/// For example, you might have a database connection or external service client that you
/// need to build beforehand and inject into other Dependencies.
///
/// 2. Non-instances:
///
/// Are types that has no need to be pre-created. Instead, you register the type itself,
/// and the container will use the `Injectable` trait to build them when needed, resolving
/// their dependencies automatically.
///
///

pub struct DependencyContainer {
    pub(crate) instances: HashMap<TypeId, Dependency>,
    pub(crate) dependency_builders: HashMap<TypeId, DependencyBuilder>,
    pub(crate) dependency_graph: HashMap<TypeId, Vec<TypeId>>,
}

impl DependencyContainer {
    pub fn builder() -> Self {
        Self {
            instances: HashMap::new(),
            dependency_builders: HashMap::new(),
            dependency_graph: HashMap::new(),
        }
    }

    pub fn register<T>(mut self) -> Self
    where
        T: Injectable + Send + Sync + 'static,
    {
        let type_id = TypeId::of::<T>();
        let type_name = std::any::type_name::<T>();

        let dependency_builder = Box::new(move |state: &State| {
            T::build(state)
                .map(|instance| Arc::new(instance) as Dependency)
                .map_err(|e| DependencyInjectionError::BuildFailed {
                    type_name: type_name.to_string(),
                    reason: e.to_string(),
                })
        });

        self.dependency_graph.insert(type_id, T::dependencies());
        self.dependency_builders.insert(type_id, dependency_builder);

        self
    }

    pub fn register_instance<T>(mut self, instance: T) -> Self
    where
        T: Send + Sync + 'static,
    {
        self.instances.insert(TypeId::of::<T>(), Arc::new(instance));
        self
    }

    pub fn register_provider<T>(mut self, provider: T) -> Self
    where
        T: Provider,
    {
        self.instances.insert(TypeId::of::<T>(), Arc::new(provider));
        self
    }

    pub fn build(self) -> Self {
        self
    }

    pub(crate) fn build_all(
        &self,
        state: &State,
    ) -> Result<(), DependencyInjectionError> {
        let mut built = HashSet::new();

        // First. register all the provided instances

        for (type_id, instance) in &self.instances {
            state
                .insert_dependency(*type_id, instance.clone())
                .map_err(|e| DependencyInjectionError::StateError {
                    type_name: format!("{:?}", type_id),
                    source: e,
                })?;
            built.insert(*type_id);
        }

        // Then, build the rest based on dependencies in
        // topological order.

        // If a type_id is already built, skip it (Dep already built).

        for type_id in self.dependency_graph.keys() {
            self.build_recursive(type_id, state, &mut built)?;
        }

        Ok(())
    }

    fn build_recursive(
        &self,
        type_id: &TypeId,
        state: &State,
        built: &mut HashSet<TypeId>,
    ) -> Result<(), DependencyInjectionError> {
        if built.contains(type_id) {
            return Ok(());
        }

        // Explore to all the dependencies first
        // and for each dependency, invoke build_recursive
        // to ensure they are built before building the current type.

        if let Some(deps) = self.dependency_graph.get(type_id) {
            for dep_id in deps {
                self.build_recursive(dep_id, state, built)?;
            }
        }

        if let Some(builder) = self.dependency_builders.get(type_id) {
            state
                .insert_dependency(*type_id, builder(state)?)
                .map_err(|e| DependencyInjectionError::StateError {
                    type_name: format!("{:?}", type_id),
                    source: e,
                })?;

            built.insert(*type_id);
        }

        Ok(())
    }
}
