mod controller;
pub use controller::NonControllerModule;

use std::any::TypeId;

use crate::{
    core::{Config, DependencyContainer},
    web::Controller,
};

#[allow(async_fn_in_trait)]
pub trait Module {
    type Controller: Controller;

    fn register_components(_: &DependencyContainer) {}
    async fn register_providers(_: &Config, _: &DependencyContainer) {}

    #[doc(hidden)]
    fn is_controller_module() -> bool {
        TypeId::of::<Self::Controller>() != TypeId::of::<NonControllerModule>()
    }
}
