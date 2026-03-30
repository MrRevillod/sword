#[cfg(test)]
mod request {
    mod cookies;
    mod multipart;
    mod query;
    mod stream;
}

#[cfg(test)]
mod middlewares {
    mod built_in;
    mod controller_level;
    mod cors_socketio;
    mod handler_level;
    mod helmet;
    mod socketio_configured;
}

#[cfg(test)]
mod application {
    mod config;
    mod di;
}

#[cfg(test)]
pub mod utils;
