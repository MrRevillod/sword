use std::any::TypeId;

use axum::Router;

use crate::{
    core::{
        Build, Config, DependencyContainer, DependencyInjectionError, HasDeps, State,
    },
    web::{Controller, ControllerBuilder},
};

#[allow(async_fn_in_trait)]
pub trait Module<C: Controller = DefaultController> {
    fn register_components(_: &mut DependencyContainer) {}

    async fn register_providers(_: &Config, _: &State, _: &mut DependencyContainer) {
    }

    #[doc(hidden)]
    fn controller() -> TypeId {
        TypeId::of::<C>()
    }

    #[doc(hidden)]
    fn router_factory() -> Option<fn(State) -> Router> {
        let controller_type_id = TypeId::of::<C>();
        let default_controller_type_id = TypeId::of::<DefaultController>();

        if controller_type_id != default_controller_type_id {
            Some(C::router)
        } else {
            None
        }
    }
}

pub struct DefaultController;

impl Controller for DefaultController {
    fn router(_state: State) -> Router {
        Router::new()
    }
}

impl Build for DefaultController {
    type Error = DependencyInjectionError;
    fn build(_: &State) -> Result<Self, Self::Error>
    where
        Self: Sized,
    {
        Ok(DefaultController)
    }
}

impl ControllerBuilder for DefaultController {
    fn base_path() -> &'static str {
        "/"
    }

    fn apply_middlewares(router: Router, _: State) -> Router {
        router
    }
}

impl HasDeps for DefaultController {
    fn deps() -> Vec<TypeId> {
        Vec::new()
    }
}
