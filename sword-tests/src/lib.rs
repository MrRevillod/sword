#[cfg(test)]
mod request {
    mod cookies;
    mod multipart;
    mod query;
}

#[cfg(test)]
mod middlewares {
    mod built_in;
    mod controller_level;
    mod handler_level;
    mod helmet;
}

#[cfg(test)]
mod application {
    mod config;
    mod di;
    mod prefix;
    mod versioning;
}

#[cfg(test)]
pub mod utils;
