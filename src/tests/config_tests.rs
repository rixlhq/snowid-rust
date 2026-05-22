//! Configuration tests

#[cfg(test)]
mod tests {
    use crate::SnowID;
    use crate::config::{SnowIDConfig, SnowIDConfigError};

    #[test]
    fn test_valid_node_bits() {
        for bits in 6..=16 {
            let config = SnowIDConfig::builder().node_bits(bits).unwrap().build();
            assert_eq!(config.node_bits(), bits);
            assert_eq!(config.sequence_bits(), SnowID::TOTAL_NODE_AND_SEQUENCE_BITS - bits);
        }
    }

    #[test]
    fn test_node_bits_ok() {
        let cfg = SnowIDConfig::builder().node_bits(12).unwrap().build();
        assert_eq!(cfg.node_bits(), 12);
    }

    #[test]
    fn test_node_bits_err() {
        let err = SnowIDConfig::builder().node_bits(5).unwrap_err();
        assert_eq!(err, SnowIDConfigError::InvalidNodeBits { bits: 5 });
    }

    #[test]
    fn test_custom_config() {
        let config = SnowIDConfig::builder().node_bits(12).unwrap().epoch(1640995200000).build();

        assert_eq!(config.node_bits(), 12);
        assert_eq!(config.sequence_bits(), 10);
        assert_eq!(config.epoch(), 1640995200000);
    }

    #[test]
    fn test_default_config() {
        let config = SnowIDConfig::default();
        assert_eq!(config.node_bits(), 10);
        assert_eq!(config.sequence_bits(), 12);
    }

    #[test]
    fn test_bit_config() {
        let config = SnowIDConfig::default();
        assert_eq!(config.max_sequence_id(), 0xFFF);
        assert_eq!(config.max_node_id(), 0x3FF);
    }
}
