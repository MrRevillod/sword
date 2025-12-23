mod expand;
mod generation;

mod handlers {
    mod expand;
    mod generation;
    mod parsing;

    pub use expand::expand_socketio_handlers;
}

pub use expand::expand_socketio_adapter;
pub use handlers::expand_socketio_handlers;
