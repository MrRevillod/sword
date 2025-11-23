use serde::Deserialize;
use tower_http::services::{ServeDir, ServeFile};

use crate::core::{ConfigItem, middlewares::BodyLimit as ChunkSize};

#[derive(Clone, Deserialize)]
pub struct ServeDirConfig {
    pub enabled: bool,
    pub static_dir: String,
    pub router_path: String,
    pub compression_algorithm: Option<String>,
    pub chunk_size: Option<ChunkSize>,
    pub not_found_file: Option<String>,
}

pub struct ServeDirMiddleware;

impl ServeDirMiddleware {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(config: ServeDirConfig) -> ServeDir<ServeFile> {
        let mut fallback = ServeFile::new(format!("{}/404.html", config.static_dir));

        if let Some(not_found_file) = config.not_found_file {
            fallback =
                ServeFile::new(format!("{}/{not_found_file}", config.static_dir));
        }

        let mut layer = ServeDir::new(config.static_dir).fallback(fallback);

        if let Some(algorithm) = config.compression_algorithm {
            match algorithm.as_str() {
                "br" => {
                    layer = layer.precompressed_br();
                }
                "gzip" => {
                    layer = layer.precompressed_gzip();
                }
                "deflate" => {
                    layer = layer.precompressed_deflate();
                }
                "zstd" => {
                    layer = layer.precompressed_zstd();
                }
                _ => {}
            }
        }

        if let Some(ChunkSize { parsed, .. }) = config.chunk_size {
            layer = layer.with_buf_chunk_size(parsed);
        }

        layer
    }
}

impl ConfigItem for ServeDirConfig {
    fn toml_key() -> &'static str {
        "serve_dir"
    }
}
