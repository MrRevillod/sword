use serde::{Deserialize, Serialize};
use sword_layers::{
    DisplayConfig, body_limit::BodyLimitConfig, compression::CompressionConfig,
    cors::CorsConfig, req_timeout::RequestTimeoutConfig, servedir::ServeDirConfig,
};

use crate::core::{ConfigItem, ConfigRegistrar};

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(default)]
pub struct MiddlewaresConfig {
    pub body_limit: BodyLimitConfig,
    pub request_timeout: RequestTimeoutConfig,
    pub compression: CompressionConfig,
    pub cors: CorsConfig,
    pub serve_dir: ServeDirConfig,
}

impl DisplayConfig for MiddlewaresConfig {
    fn display(&self) {
        self.body_limit.display();
        self.request_timeout.display();
        self.compression.display();
        self.cors.display();
        self.serve_dir.display();
    }
}

impl ConfigItem for MiddlewaresConfig {
    fn toml_key() -> &'static str {
        "middlewares"
    }
}

const _: () = {
    inventory::submit! {
        ConfigRegistrar::new(|config, state| {
            MiddlewaresConfig::register(config, state)
        })
    }
};
