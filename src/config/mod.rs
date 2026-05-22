//! Configuration for SnowID generator
//!
#![allow(clippy::cast_possible_truncation)]

mod builder;

use std::error::Error;
use std::fmt;

pub use builder::SnowIDConfigBuilder;
use builder::{DEFAULT_CUSTOM_EPOCH, DEFAULT_NODE_BITS};

use crate::SnowID;

/// Errors related to `SnowIDConfig` builder validation
#[derive(Debug, Clone, PartialEq)]
pub enum SnowIDConfigError {
    /// Provided node bits are out of the supported range [6, 16]
    InvalidNodeBits { bits: u8 },
}

impl fmt::Display for SnowIDConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::InvalidNodeBits { bits } => {
                write!(f, "Node bits {} must be between 6 and 16", bits)
            },
        }
    }
}

impl Error for SnowIDConfigError {}

/// Configuration for SnowID generator
/// Copy-optimized with const-evaluable fields
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct SnowIDConfig {
    node_bits: u8,
    custom_epoch: u64,
    timestamp_shift: u8,
    node_shift: u8,
    timestamp_mask: u64,
    node_mask: u16,
    sequence_mask: u16,
}

impl SnowIDConfig {
    /// Calculate mask for given number of bits
    pub(crate) const fn calculate_mask(bits: u8) -> u16 {
        ((1u32 << bits) - 1) as u16
    }

    /// Create new SnowIDConfig with given node bits
    const fn new(node_bits: u8, custom_epoch: u64) -> Self {
        let sequence_bits = SnowID::TOTAL_NODE_AND_SEQUENCE_BITS - node_bits;
        Self {
            node_bits,
            custom_epoch,
            timestamp_shift: SnowID::TOTAL_NODE_AND_SEQUENCE_BITS,
            node_shift: sequence_bits,
            timestamp_mask: (1u64 << SnowID::TIMESTAMP_BITS) - 1,
            node_mask: Self::calculate_mask(node_bits),
            sequence_mask: Self::calculate_mask(sequence_bits),
        }
    }

    /// Create config from builder
    pub(crate) fn from_builder(b: SnowIDConfigBuilder) -> Self {
        Self::new(b.node_bits, b.custom_epoch)
    }

    /// Create a new configuration builder
    #[must_use]
    pub const fn builder() -> SnowIDConfigBuilder {
        SnowIDConfigBuilder::new()
    }

    #[inline(always)]
    #[must_use]
    pub const fn epoch(&self) -> u64 {
        self.custom_epoch
    }

    #[inline(always)]
    #[must_use]
    pub const fn node_bits(&self) -> u8 {
        self.node_bits
    }

    #[inline(always)]
    #[must_use]
    pub const fn sequence_bits(&self) -> u8 {
        SnowID::TOTAL_NODE_AND_SEQUENCE_BITS - self.node_bits
    }

    #[inline(always)]
    #[must_use]
    pub const fn max_node_id(&self) -> u16 {
        self.node_mask
    }

    #[inline(always)]
    #[must_use]
    pub const fn max_sequence_id(&self) -> u16 {
        self.sequence_mask
    }

    #[inline(always)]
    pub(crate) const fn timestamp_shift(&self) -> u8 {
        self.timestamp_shift
    }

    #[inline(always)]
    pub(crate) const fn node_shift(&self) -> u8 {
        self.node_shift
    }

    #[inline(always)]
    pub(crate) const fn timestamp_mask(&self) -> u64 {
        self.timestamp_mask
    }

    #[inline(always)]
    pub(crate) const fn node_mask(&self) -> u16 {
        self.node_mask
    }

    #[inline(always)]
    pub(crate) const fn sequence_mask(&self) -> u16 {
        self.sequence_mask
    }
}

impl Default for SnowIDConfig {
    fn default() -> Self {
        Self::new(DEFAULT_NODE_BITS, DEFAULT_CUSTOM_EPOCH)
    }
}
