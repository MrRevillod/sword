mod config;
#[doc(hidden)]
pub mod error;
mod injectables;
mod state;

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
