use crate::internal::{ConfigItem, ConfigRegistrar};
use serde::{Deserialize, Serialize};
use sword_layers::prelude::{
    BodyLimitConfig, CompressionConfig, CorsConfig, DisplayConfig,
    RequestTimeoutConfig, ServeDirConfig, SocketIoServerConfig,
};

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(default)]
pub struct MiddlewaresConfig {
    #[serde(rename = "body-limit")]
    pub body_limit: BodyLimitConfig,

    #[serde(rename = "request-timeout")]
    pub request_timeout: RequestTimeoutConfig,

    pub compression: CompressionConfig,
    pub cors: CorsConfig,
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
    fn toml_key() -> &'static str {
        "middlewares"
    }

    fn register(
        config: &crate::core::Config,
        state: &crate::core::State,
    ) -> Result<(), crate::core::ConfigError> {
        state.insert(config.get_or_default::<Self>());
        Ok(())
    }
}

impl ConfigItem for ServeDirConfig {
    fn toml_key() -> &'static str {
        "serve-dir"
    }

    fn register(
        config: &crate::core::Config,
        state: &crate::core::State,
    ) -> Result<(), crate::core::ConfigError> {
        state.insert(config.get_or_default::<Self>());
        Ok(())
    }
}

impl ConfigItem for SocketIoServerConfig {
    fn toml_key() -> &'static str {
        "socketio-server"
    }

    fn register(
        config: &crate::core::Config,
        state: &crate::core::State,
    ) -> Result<(), crate::core::ConfigError> {
        println!("Registering SocketIoServerConfig");
        state.insert(config.get_or_default::<Self>());
        Ok(())
    }
}

const _: () = {
    inventory::submit! {
        ConfigRegistrar::new(|config, state| {
            MiddlewaresConfig::register(config, state)
        })
    }

    inventory::submit! {
        ConfigRegistrar::new(|config, state| {
            ServeDirConfig::register(config, state)
        })
    }

    inventory::submit! {
        ConfigRegistrar::new(|config, state| {
            SocketIoServerConfig::register(config, state)
        })
    }
};
