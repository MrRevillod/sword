mod generation;
mod on_handler;

pub(crate) use generation::generate_socketio_controller_builder;

pub use on_handler::expand_on_handler;
