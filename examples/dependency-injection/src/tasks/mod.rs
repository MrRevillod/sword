mod controller;
mod entity;
mod middleware;
mod repository;

pub use controller::TasksController;
pub use entity::Task;
pub use repository::TaskRepository;

use sword::prelude::*;

pub struct TasksModule;

impl Module<TasksController> for TasksModule {
    fn register_components(container: &mut DependencyContainer) {
        container.register_component::<TaskRepository>();
    }
}
