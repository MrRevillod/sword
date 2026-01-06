mod config;
mod injectables;
pub mod layers;
mod state;

pub use injectables::*;

pub use config::*;
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
