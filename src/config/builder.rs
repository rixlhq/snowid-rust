//! SnowIDConfig builder for constructing configuration

use super::{SnowIDConfig, SnowIDConfigError};

/// Default configuration values
pub(super) const DEFAULT_NODE_BITS: u8 = 10;
pub(super) const DEFAULT_CUSTOM_EPOCH: u64 = 1704067200000; // January 1, 2024 UTC

/// Builder for SnowIDConfig
#[derive(Debug)]
pub struct SnowIDConfigBuilder {
    pub(super) node_bits: u8,
    pub(super) custom_epoch: u64,
}

impl SnowIDConfigBuilder {
    /// Create a new SnowIDConfigBuilder with default values
    pub const fn new() -> Self {
        Self {
            node_bits: DEFAULT_NODE_BITS,
            custom_epoch: DEFAULT_CUSTOM_EPOCH,
        }
    }

    /// Set the number of bits for node ID (6-16)
    /// Sequence bits will be automatically set to (22 - node_bits)
    pub fn node_bits(mut self, bits: u8) -> Result<Self, SnowIDConfigError> {
        if !(6..=16).contains(&bits) {
            return Err(SnowIDConfigError::InvalidNodeBits { bits });
        }
        self.node_bits = bits;
        Ok(self)
    }

    /// Set a custom epoch timestamp in milliseconds
    pub const fn epoch(mut self, epoch: u64) -> Self {
        self.custom_epoch = epoch;
        self
    }

    /// Build the final SnowIDConfig
    pub fn build(self) -> SnowIDConfig {
        SnowIDConfig::new(self.node_bits, self.custom_epoch)
    }
}

impl Default for SnowIDConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}
