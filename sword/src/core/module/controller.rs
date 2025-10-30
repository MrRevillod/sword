use axum::Router;
use std::any::TypeId;

use crate::{
    core::{Build, DependencyInjectionError, HasDeps, State},
    web::{Controller, ControllerBuilder},
};

pub struct NonControllerModule;

impl Controller for NonControllerModule {
    fn router(_state: State) -> Router {
        Router::new()
    }
}

impl Build for NonControllerModule {
    type Error = DependencyInjectionError;
    fn build(_: &State) -> Result<Self, Self::Error>
    where
        Self: Sized,
    {
        Ok(NonControllerModule)
    }
}

impl ControllerBuilder for NonControllerModule {
    fn base_path() -> &'static str {
        "/"
    }

    fn apply_middlewares(router: Router, _: State) -> Router {
        router
    }
}

impl HasDeps for NonControllerModule {
    fn deps() -> Vec<TypeId> {
        Vec::new()
    }
}
