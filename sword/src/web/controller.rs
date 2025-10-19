use crate::core::{DependencyInjectionError, State};
use axum::Router as AxumRouter;

pub use sword_macros::{controller, delete, get, patch, post, put, routes};

pub trait Controller: ControllerBuilder {
    fn router(state: State) -> AxumRouter;
}

pub trait ControllerBuilder {
    fn base_path() -> &'static str;
    fn apply_middlewares(router: AxumRouter, state: State) -> AxumRouter;

    fn build(state: &State) -> Result<Self, DependencyInjectionError>
    where
        Self: Sized;
}
