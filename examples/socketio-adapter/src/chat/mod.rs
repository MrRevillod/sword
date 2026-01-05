mod adapter;
mod dtos;
mod entity;

use adapter::ChatAdapter;

pub use dtos::IncommingMessageDto;
pub use entity::Message;

use sword::prelude::*;

pub struct ChatModule;

impl Module for ChatModule {
    fn register_adapters(adapters: &AdapterRegistry) {
        adapters.register::<ChatAdapter>();
    }
}
