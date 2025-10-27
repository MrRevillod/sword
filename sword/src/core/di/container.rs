use std::{
    any::{TypeId, type_name},
    collections::{HashMap, HashSet},
    sync::Arc,
};

use crate::core::{
    Component, Provider, State,
    di::{Dependency, DependencyBuilder, DependencyInjectionError},
};

/// A container for managing dependencies and their builders.
///
/// Basically it support two types of registrations:
///
/// 1. Providers:
///
/// Providers are pre-created objects that you want to register directly into the container.
/// For example, you might have a database connection or external service client that you
/// need to build beforehand and inject into other Dependencies.
///
/// 2. Components:
///
/// Are types that has no need to be pre-created. Instead, you register the type itself,
/// and the container will use the `Component` trait to build them when needed, resolving
/// their dependencies automatically.
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

    /// Registers an injectable component.
    ///
    /// The component must implement the `Component` trait. The container will store
    /// a builder function and the component's dependency list for later construction
    /// during the `build_all` phase.
    ///
    /// Dependencies are resolved automatically using topological sorting based on
    /// the dependency graph.
    pub fn register_component<T: Component>(&mut self) {
        let type_id = TypeId::of::<T>();
        let type_name = type_name::<T>();

        let dependency_builder = Box::new(move |state: &State| {
            T::build(state)
                .map(|instance| Arc::new(instance) as Dependency)
                .map_err(|e| DependencyInjectionError::BuildFailed {
                    type_name: type_name.to_string(),
                    reason: e.to_string(),
                })
        });

        self.dependency_graph.insert(type_id, T::deps());
        self.dependency_builders.insert(type_id, dependency_builder);
    }

    /// Registers a pre-built dependency provider.
    ///
    /// Providers are instances that have already been constructed and are ready
    /// to be injected into other components. Typical use cases include database
    /// connections, HTTP clients, or external service configurations that cannot
    /// be auto-constructed from the State.
    pub fn register_provider<T>(&mut self, provider: T)
    where
        T: Provider,
    {
        self.instances.insert(TypeId::of::<T>(), Arc::new(provider));
    }

    pub fn build(self) -> Self {
        self
    }

    /// Builds all registered components in dependency order.
    ///
    /// This internal method performs the following steps:
    /// 1. Registers all provider instances in the State
    /// 2. Performs topological sorting on the dependency graph
    /// 3. Constructs components recursively in the correct order
    /// 4. Detects circular dependencies and returns an error if found
    ///
    /// This method is called internally during application initialization.
    pub(crate) fn build_all(
        &self,
        state: &State,
    ) -> Result<(), DependencyInjectionError> {
        let mut built = HashSet::new();
        let mut visiting = HashSet::new();

        // First. register all the provided instances

        for (type_id, instance) in &self.instances {
            state
                .insert_dependency(*type_id, Arc::clone(instance))
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
            self.build_recursive(type_id, state, &mut built, &mut visiting)?;
        }

        Ok(())
    }

    /// Recursively builds a component and its dependencies.
    ///
    /// This method implements depth-first traversal of the dependency graph:
    /// - Skips already built components
    /// - Detects circular dependencies using a visiting set
    /// - Recursively builds all dependencies before the component itself
    /// - Invokes the builder function and stores the result in State
    fn build_recursive(
        &self,
        type_id: &TypeId,
        state: &State,
        built: &mut HashSet<TypeId>,
        visiting: &mut HashSet<TypeId>,
    ) -> Result<(), DependencyInjectionError> {
        if built.contains(type_id) {
            return Ok(());
        }

        if visiting.contains(type_id) {
            return Err(DependencyInjectionError::CircularDependency {
                type_name: std::any::type_name::<()>().to_string(),
            });
        }

        visiting.insert(*type_id);

        // Explore to all the dependencies first
        // and for each dependency, invoke build_recursive
        // to ensure they are built before building the current type.

        if let Some(deps) = self.dependency_graph.get(type_id) {
            for dep_id in deps {
                self.build_recursive(dep_id, state, built, visiting)?;
            }
        }

        visiting.remove(type_id);

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
