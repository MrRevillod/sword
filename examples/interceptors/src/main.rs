mod interceptors;
mod controllers {
    pub mod socketio;
    pub mod web;
}

use controllers::socketio::EventsController;
use controllers::web::ExampleRestController;
use sword::prelude::*;

pub struct ExampleModule;

impl Module for ExampleModule {
    fn register_controllers(controllers: &ControllerRegistry) {
        controllers.register::<EventsController>();
        controllers.register::<ExampleRestController>();
    }
}

#[sword::main]
async fn main() {
    let app = Application::from_config_path("Config.toml")
        .with_module::<ExampleModule>()
        .build();

    app.run().await;
}
