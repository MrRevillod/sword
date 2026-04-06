mod controller;
mod dtos;
mod entity;
mod repository;

pub mod proto {
    tonic::include_proto!("users");
    pub const FILE_DESCRIPTOR_SET: &[u8] = tonic::include_file_descriptor_set!("users_descriptor");
    pub use user_service_server::*;
}

::sword::internal::inventory::submit! {
    ::sword::internal::grpc::GrpcReflectionDescriptorRegistrar {
        encoded_file_descriptor_set: proto::FILE_DESCRIPTOR_SET,
    }
}

use controller::UsersController;

pub use dtos::{CreateUserDto, UpdateUserDto};
pub use entity::User;
pub use repository::UserRepository;

use sword::prelude::*;

pub struct UsersModule;

impl Module for UsersModule {
    fn register_components(components: &ComponentRegistry) {
        components.register::<UserRepository>();
    }

    fn register_controllers(controllers: &ControllerRegistry) {
        controllers.register::<UsersController>();
    }
}
