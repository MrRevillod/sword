#[macro_use]
pub(crate) mod macros;
pub mod expand;
pub mod generation;

pub mod routes {
    mod expand;
    mod generation;
    mod parsing;

    pub use expand::*;
    pub use generation::*;
    pub use parsing::*;
}

pub use expand::expand_controller;
pub use routes::expand_controller_routes;
