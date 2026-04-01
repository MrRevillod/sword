use crate::DisplayConfig;
use serde::{Deserialize, Serialize};
use tower_http::compression::CompressionLayer as TowerCompressionLayer;

pub struct CompressionLayer;

/// # Compression Layer
///
/// This struct represents the Compression Layer which
/// enables response compression based on the provided configuration.
///
/// The layer is a wrapper around `tower_http::compression::CompressionLayer`.
/// Compress response bodies of the underlying service.
///
/// This uses the Accept-Encoding header to pick an appropriate encoding
/// and adds the Content-Encoding header to responses.

/// ## Compression Configuration
///
/// This configuration allows you to enable or disable compression
/// and specify which algorithms to use.

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(default)]
pub struct CompressionConfig {
    /// Whether to display the configuration details.
    pub display: bool,
    /// A list of strings representing the compression algorithms to use
    /// (e.g., "gzip", "deflate", "br", "zstd
    pub algorithms: Vec<String>,
}

impl From<CompressionConfig> for TowerCompressionLayer {
    fn from(config: CompressionConfig) -> Self {
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

        layer
    }
}

impl DisplayConfig for CompressionConfig {
    fn display(&self) {
        if self.display {
            tracing::info!(
                target: "sword.layers.compression",
                algorithms = ?self.algorithms,
            );
        }
    }
}
