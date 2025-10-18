use crate::{core::State, errors::DependencyInjectionError};
use axum::Router as AxumRouter;

pub trait Controller: ControllerBuilder {
    fn router(state: State) -> AxumRouter;
}

pub trait ControllerBuilder {
    fn base_path() -> &'static str;

    fn apply_controller_middlewares(
        router: AxumRouter,
        app_state: State,
    ) -> AxumRouter;

    fn build(state: &State) -> Result<Self, DependencyInjectionError>
    where
        Self: Sized;
}
