use serde::{Deserialize, Serialize};
use tower_http::compression::CompressionLayer as TowerCompressionLayer;

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct CompressionConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub algorithms: Vec<String>,
}

pub struct CompressionLayer;

impl CompressionLayer {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(config: CompressionConfig) -> Option<TowerCompressionLayer> {
        if !config.enabled {
            return None;
        }

        let mut layer = TowerCompressionLayer::new();

        for algorithm in &config.algorithms {
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

    #[serde(default = "default_display")]
    pub display: bool,
}

fn default_display() -> bool {
    false
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
