//! Core SnowID generator implementation
//!
//! Split into modules for testability:
//! - `state` - Combined atomic state (timestamp + sequence)
//! - `time` - Wall-clock time utilities
//! - `generate` - ID generation logic

mod base62_methods;
mod generate;
mod state;
mod time;

pub use generate::TryGenerateError;

use std::sync::atomic::AtomicU64;

use crate::config::SnowIDConfig;
use crate::error::SnowIDError;
use crate::extractor::SnowIDExtractor;

use time::{time_since_epoch, unix_time_ms};

/// Main ID generator with cache-line alignment
#[derive(Debug)]
#[repr(align(64))]
pub struct SnowID {
    // === Hot path fields ===
    pub(crate) state: AtomicU64,
    node_prefix: u64,
    pub(crate) max_seq: u16,
    ts_shift: u8,
    ts_mask: u64,
    epoch: u64,

    // === Cold path fields ===
    pub node_id: u16,
    pub config: SnowIDConfig,
    pub extract: SnowIDExtractor,
}

impl SnowID {
    pub const TIMESTAMP_BITS: u32 = 42;
    pub const TOTAL_NODE_AND_SEQUENCE_BITS: u8 = 22;
    pub const MAX_TIMESTAMP: u64 = (1u64 << Self::TIMESTAMP_BITS) - 1;

    /// Create with default configuration
    pub fn new(node_id: u16) -> Result<Self, SnowIDError> {
        Self::with_config(node_id, SnowIDConfig::default())
    }

    /// Create with custom configuration
    pub fn with_config(node_id: u16, config: SnowIDConfig) -> Result<Self, SnowIDError> {
        Self::validate_node_id(node_id, &config)?;
        Ok(Self::build(node_id, config))
    }

    fn validate_node_id(node_id: u16, config: &SnowIDConfig) -> Result<(), SnowIDError> {
        let max = config.max_node_id();
        if node_id > max {
            return Err(SnowIDError::InvalidNodeId { node_id, max });
        }
        Self::validate_epoch(config)?;
        Ok(())
    }

    fn validate_epoch(config: &SnowIDConfig) -> Result<(), SnowIDError> {
        let now = unix_time_ms();
        let epoch = config.epoch();
        if epoch > now || now - epoch > Self::MAX_TIMESTAMP {
            return Err(SnowIDError::InvalidEpoch { epoch, now, max_age_ms: Self::MAX_TIMESTAMP });
        }
        Ok(())
    }

    const fn build(node_id: u16, config: SnowIDConfig) -> Self {
        Self {
            state: AtomicU64::new(0),
            node_prefix: Self::compute_node_prefix(node_id, &config),
            max_seq: config.max_sequence_id(),
            ts_shift: config.timestamp_shift(),
            ts_mask: config.timestamp_mask(),
            epoch: config.epoch(),
            node_id,
            config,
            extract: SnowIDExtractor::new(config),
        }
    }

    #[inline(always)]
    const fn compute_node_prefix(node_id: u16, config: &SnowIDConfig) -> u64 {
        (node_id as u64) << config.node_shift()
    }

    #[inline(always)]
    pub(crate) fn now_ms(&self) -> u64 {
        time_since_epoch(self.epoch)
    }

    #[inline(always)]
    pub(crate) const fn assemble_id(&self, timestamp: u64, sequence: u16) -> u64 {
        ((timestamp & self.ts_mask) << self.ts_shift) | self.node_prefix | (sequence as u64)
    }
}
