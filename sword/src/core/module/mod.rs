mod gateway;

use crate::core::{Config, DependencyContainer};

pub use gateway::*;

#[allow(async_fn_in_trait)]
pub trait Module {
    fn register_gateways(_: &GatewayRegistry) {}
    fn register_components(_: &DependencyContainer) {}
    async fn register_providers(_: &Config, _: &DependencyContainer) {}
}
