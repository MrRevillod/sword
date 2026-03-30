#![cfg(feature = "socketio-controllers")]

mod expand;
mod parse;

pub use expand::generate_socketio_controller_builder;
pub use parse::expand_on_handler;
