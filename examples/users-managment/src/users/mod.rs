mod adapters {
    pub mod rest;
    pub mod socketio;
}

mod dtos;
mod entity;
mod repository;

use adapters::rest::UsersController;
use adapters::socketio::UserMessagesAdapter;

pub use dtos::{CreateUserDto, UpdateUserDto};
pub use entity::User;
pub use repository::UserRepository;

use sword::prelude::*;

pub struct UsersModule;

impl Module for UsersModule {
    fn register_components(components: &ComponentRegistry) {
        components.register::<UserRepository>();
    }

    fn register_adapters(adapters: &AdapterRegistry) {
        adapters.register::<UsersController>();
        adapters.register::<UserMessagesAdapter>();
    }
}
