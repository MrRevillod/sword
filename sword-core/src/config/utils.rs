use byte_unit::Byte;
use duration_str::parse as parse_duration;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use std::time::Duration;

/// Time duration configuration with raw string representation
///
/// This type allows deserializing human-readable time durations from configuration files.
///
/// # Examples
///
/// ```toml
/// timeout = "30s"
/// interval = "1h 30m"
/// ```
#[derive(Debug, Clone, Serialize)]
pub struct TimeConfig {
    pub parsed: Duration,
    pub raw: String,
}

impl<'de> Deserialize<'de> for TimeConfig {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::Error;

        let raw = String::deserialize(deserializer)?;
        let parsed = parse_duration(&raw).map_err(Error::custom)?;

        Ok(TimeConfig { parsed, raw })
    }
}

/// Byte size configuration with raw string representation
///
/// This type allows deserializing human-readable byte sizes from configuration files.
///
/// # Examples
///
/// ```toml
/// max_size = "10MB"
/// buffer_size = "4KiB"
/// ```
#[derive(Debug, Clone, Serialize)]
pub struct ByteConfig {
    pub parsed: usize,
    pub raw: String,
}

impl<'de> Deserialize<'de> for ByteConfig {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::Error;

        let raw = String::deserialize(deserializer)?;
        let byte = Byte::from_str(&raw).map_err(Error::custom)?;
        let parsed = byte.as_u64() as usize;

        Ok(ByteConfig { parsed, raw })
    }
}
