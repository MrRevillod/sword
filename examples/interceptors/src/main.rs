mod interceptors;
mod adapters {
    pub mod http;
    pub mod socketio;
}

use adapters::http::ExampleRestController;
use adapters::socketio::EventsHandler;
use sword::prelude::*;

pub struct ExampleModule;

impl Module for ExampleModule {
    fn register_adapters(adapters: &AdapterRegistry) {
        adapters.register::<EventsHandler>();
        adapters.register::<ExampleRestController>();
    }
}

#[sword::main]
async fn main() {
    let app = Application::from_config_path("Config.toml")
        .with_module::<ExampleModule>()
        .build();

    app.run().await;
}
