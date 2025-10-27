mod shared {
    pub mod database;
}

mod tasks;

use dotenv::dotenv;
use sword::prelude::*;

use crate::{shared::database::DatabaseModule, tasks::TasksModule};

#[sword::main]
async fn main() {
    dotenv().ok();

    let app = Application::builder()
        .with_module::<DatabaseModule, _>()
        .with_module::<TasksModule, _>()
        .build();

    app.run().await;
}
