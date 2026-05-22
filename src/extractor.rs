#![allow(clippy::cast_possible_truncation)]

use crate::config::SnowIDConfig;

/// SnowID component extractor
#[derive(Debug, Copy, Clone)]
pub struct SnowIDExtractor {
    config: SnowIDConfig,
}

impl SnowIDExtractor {
    /// Create a new SnowID extractor with the given configuration
    pub(crate) const fn new(config: SnowIDConfig) -> Self {
        Self { config }
    }

    /// Extract timestamp component from a SnowID
    #[inline(always)]
    #[must_use]
    pub const fn timestamp(&self, id: u64) -> u64 {
        (id >> self.config.timestamp_shift()) & self.config.timestamp_mask()
    }

    /// Extract node component from a SnowID
    #[inline(always)]
    #[must_use]
    pub fn node(&self, id: u64) -> u16 {
        ((id >> self.config.node_shift()) & u64::from(self.config.node_mask())) as u16
    }

    /// Extract sequence component from a SnowID
    #[inline(always)]
    #[must_use]
    pub fn sequence(&self, id: u64) -> u16 {
        (id & u64::from(self.config.sequence_mask())) as u16
    }

    /// Decompose SnowID into its components: timestamp, node ID, and sequence
    #[inline]
    #[must_use]
    pub fn decompose(&self, id: u64) -> (u64, u16, u16) {
        (self.timestamp(id), self.node(id), self.sequence(id))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const fn create_snowid(config: SnowIDConfig, timestamp: u64, node: u16, sequence: u16) -> u64 {
        ((timestamp & config.timestamp_mask()) << config.timestamp_shift()) | ((node as u64) << config.node_shift()) | (sequence as u64)
    }

    #[test]
    fn test_decompose() {
        let config = SnowIDConfig::default();
        let extractor = SnowIDExtractor::new(config);

        let timestamp: u64 = 0x1234567;
        let node: u16 = 42;
        let sequence: u16 = 123;

        let id = create_snowid(config, timestamp, node, sequence);

        assert_eq!(extractor.timestamp(id), timestamp);
        assert_eq!(extractor.node(id), node);
        assert_eq!(extractor.sequence(id), sequence);

        let (ext_timestamp, ext_node, ext_sequence) = extractor.decompose(id);
        assert_eq!(ext_timestamp, timestamp);
        assert_eq!(ext_node, node);
        assert_eq!(ext_sequence, sequence);
    }

    #[test]
    fn test_component_boundaries() {
        let config = SnowIDConfig::default();
        let extractor = SnowIDExtractor::new(config);

        let max_timestamp = (1u64 << 42) - 1;
        let max_node_id = config.max_node_id();
        let max_sequence = config.max_sequence_id();

        let id = create_snowid(config, max_timestamp, max_node_id, max_sequence);

        assert_eq!(extractor.timestamp(id), max_timestamp);
        assert_eq!(extractor.node(id), max_node_id);
        assert_eq!(extractor.sequence(id), max_sequence);
    }
}
