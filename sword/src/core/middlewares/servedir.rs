use byte_unit::Byte;
use serde::{Deserialize, Serialize};
use tower_http::services::{ServeDir, ServeFile};

#[derive(Clone, Debug, Deserialize, Serialize, Default)]
pub struct ServeDirConfig {
    pub enabled: bool,
    pub static_dir: String,
    pub router_path: String,
    pub compression_algorithm: Option<String>,
    pub chunk_size: Option<String>,
    pub not_found_file: Option<String>,

    #[serde(default = "default_display")]
    pub display: bool,
}

fn default_display() -> bool {
    false
}

impl ServeDirConfig {
    pub fn display(&self) {
        use console::style;

        println!();
        println!("{}", style("Serve Directory Configuration:").bold());
        println!("  ↳  Enabled: {}", self.enabled);
        println!("  ↳  Static Directory: {}", self.static_dir);
        println!("  ↳  Router Path: {}", self.router_path);

        if let Some(algo) = &self.compression_algorithm {
            println!("  ↳  Compression Algorithm: {}", algo);
        }
        if let Some(chunk_size) = &self.chunk_size {
            println!("  ↳  Chunk Size: {}", chunk_size);
        }
        if let Some(not_found) = &self.not_found_file {
            println!("  ↳  Not Found File: {}", not_found);
        }
    }
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

        if let Some(chunk_size_str) = &config.chunk_size
            && let Ok(chunk_size) = chunk_size_str.parse::<Byte>()
        {
            layer = layer.with_buf_chunk_size(chunk_size.as_u64() as usize);
        }

        layer
    }
}
