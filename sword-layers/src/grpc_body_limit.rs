//! gRPC message size limiting configuration.
//!
//! This module provides a dedicated config type for gRPC server message limits,
//! mapped to tonic's per-service `max_decoding_message_size` and
//! `max_encoding_message_size` settings.

use crate::DisplayConfig;

use byte_unit::Byte;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use thisconfig::ByteConfig;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct GrpcBodyLimitConfig {
    /// Maximum allowed size for inbound decoded gRPC messages.
    #[serde(rename = "max-decoding-message-size")]
    pub max_decoding_message_size: ByteConfig,

    /// Maximum allowed size for outbound encoded gRPC messages.
    #[serde(rename = "max-encoding-message-size")]
    pub max_encoding_message_size: ByteConfig,

    /// Whether to log the configured limits at startup.
    pub display: bool,
}

impl DisplayConfig for GrpcBodyLimitConfig {
    fn display(&self) {
        if !self.display {
            return;
        }

        tracing::info!(
            target: "sword.layers.grpc.body-limit",
            max_decoding_message_size = self.max_decoding_message_size.raw,
            max_encoding_message_size = self.max_encoding_message_size.raw,
        );
    }
}

impl Default for GrpcBodyLimitConfig {
    fn default() -> Self {
        let decode_raw = "4MB".to_string();
        let encode_raw = "4MB".to_string();

        let decode_parsed = Byte::from_str(&decode_raw)
            .unwrap_or_else(|_| Byte::from_u64(4 * 1024 * 1024))
            .as_u64() as usize;

        let encode_parsed = Byte::from_str(&encode_raw)
            .unwrap_or_else(|_| Byte::from_u64(4 * 1024 * 1024))
            .as_u64() as usize;

        Self {
            max_decoding_message_size: ByteConfig {
                parsed: decode_parsed,
                raw: decode_raw,
            },
            max_encoding_message_size: ByteConfig {
                parsed: encode_parsed,
                raw: encode_raw,
            },
            display: false,
        }
    }
}
