mod controller;
mod dtos;
mod entity;
mod repository;

pub use controller::UsersController;
pub use dtos::{CreateUserDto, UpdateUserDto};
pub use entity::User;
pub use repository::UserRepository;

use sword::prelude::*;

pub struct UsersModule;

impl Module for UsersModule {
    fn register_components(container: &DependencyContainer) {
        container.register_component::<UserRepository>();
    }

    fn register_gateways(_: &GatewayRegistry) {}
}
