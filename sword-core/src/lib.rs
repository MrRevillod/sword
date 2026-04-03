mod config;
mod injectables;
mod state;

pub mod error {
    #[macro_use]
    mod macros;
    mod diagnostic;

    pub use diagnostic::{StartupDiagnostic, emit};
}

pub use config::*;
pub use injectables::*;
pub use state::*;

pub(crate) type RwMap<K, V> = parking_lot::RwLock<std::collections::HashMap<K, V>>;

#[macro_export]
macro_rules! inventory_submit {
    ( [ $( $registrar:expr ),* $(,)? ] ) => {
        $(
            const _: () = {
                inventory::submit! {
                    $registrar
                }
            };
        )*
    };
}
