use crate::{ConfigItem, ConfigRegistrar, inventory_submit};

use serde::{Deserialize, Serialize};
use sword_layers::prelude::{
    BodyLimitConfig, CompressionConfig, CorsConfig, DisplayConfig,
    RequestTimeoutConfig, ServeDirConfig,
};

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(default)]
pub struct MiddlewaresConfig {
    #[serde(rename = "body-limit")]
    pub body_limit: BodyLimitConfig,

    #[serde(rename = "request-timeout")]
    pub request_timeout: RequestTimeoutConfig,

    pub cors: CorsConfig,
    pub compression: CompressionConfig,
}

impl DisplayConfig for MiddlewaresConfig {
    fn display(&self) {
        self.body_limit.display();
        self.request_timeout.display();
        self.compression.display();
        self.cors.display();
    }
}

impl ConfigItem for MiddlewaresConfig {
    fn key() -> &'static str {
        "middlewares"
    }
}

inventory_submit! {[
    ConfigRegistrar::new(|state, config| {
        state.insert(config.get_or_default::<MiddlewaresConfig>());
    }),
    ConfigRegistrar::new(|state, config| {
        state.insert(config.get_or_default::<ServeDirConfig>());
    }),
]}
