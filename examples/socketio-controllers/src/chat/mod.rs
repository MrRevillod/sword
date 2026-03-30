mod controller;
mod dtos;
mod entity;
mod interceptors;

use controller::ChatController;

pub use dtos::IncommingMessageDto;
pub use entity::Message;

use sword::prelude::*;

pub struct ChatModule;

impl Module for ChatModule {
    fn register_controllers(controllers: &ControllerRegistry) {
        controllers.register::<ChatController>();
    }
}
