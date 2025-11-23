use crate::core::{ConfigItem, ConfigRegistrar};
use serde::{Deserialize, Serialize};
use tower_http::compression::CompressionLayer;

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct CompressionConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub algorithms: Vec<String>,
}

impl CompressionConfig {
    pub fn layer(self) -> Option<CompressionLayer> {
        if !self.enabled {
            return None;
        }

        let mut layer = CompressionLayer::new();

        for algorithm in &self.algorithms {
            match algorithm.to_lowercase().as_str() {
                "gzip" => layer = layer.gzip(true),
                "deflate" => layer = layer.deflate(true),
                "br" | "brotli" => layer = layer.br(true),
                "zstd" => layer = layer.zstd(true),
                _ => {}
            }
        }

        Some(layer)
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct CompressionMiddlewareConfig {
    #[serde(flatten)]
    pub compression: CompressionConfig,
}

impl CompressionMiddlewareConfig {
    pub fn display(&self) {
        use console::style;

        println!();
        println!("{}", style("Compression Configuration:").bold());

        if self.compression.enabled {
            if self.compression.algorithms.is_empty() {
                println!("  ↳  {}", style("No algorithms enabled").yellow());
            } else {
                println!(
                    "  ↳  Algorithms: {}",
                    self.compression.algorithms.join(", ")
                );
            }
        } else {
            println!("  ↳  {}", style("Compression: disabled").red());
        }
    }
}

impl ConfigItem for CompressionMiddlewareConfig {
    fn toml_key() -> &'static str {
        "compression"
    }
}

const _: () = {
    inventory::submit! {
        ConfigRegistrar::new(|config, state| {
            CompressionMiddlewareConfig::register(config, state)
        })
    }
};
