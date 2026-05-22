#[cfg(test)]
mod tests {
    use crate::SnowID;
    use crate::SnowIDError;
    use crate::config::SnowIDConfig;
    use crate::tests::test_utils::wall_clock_ms;

    #[test]
    fn test_invalid_node_id() {
        match SnowID::new(1024) {
            Err(SnowIDError::InvalidNodeId { node_id, max }) => {
                assert_eq!(node_id, 1024);
                assert_eq!(max, 1023);
            },
            _ => panic!("Expected InvalidNodeId error"),
        }
    }

    #[test]
    fn test_node_id_boundaries() {
        // Test minimum node ID
        let gen0 = SnowID::new(0).unwrap();
        let snowid0 = gen0.generate();
        let (_, node0, _) = gen0.extract.decompose(snowid0);
        assert_eq!(node0, 0);

        // Test maximum node ID
        let gen1023 = SnowID::new(1023).unwrap();
        let snowid1023 = gen1023.generate();
        let (_, node1023, _) = gen1023.extract.decompose(snowid1023);
        assert_eq!(node1023, 1023);
    }

    #[test]
    fn test_component_boundaries() {
        let config = SnowIDConfig::builder().node_bits(10).unwrap().epoch(0).build();

        let generator = SnowID::with_config(1023, config).unwrap();

        // Test timestamp boundaries
        let snowid = generator.generate();
        let (timestamp, _, _) = generator.extract.decompose(snowid);
        assert!(timestamp > 0);
        assert!(timestamp < (1u64 << SnowID::TIMESTAMP_BITS));

        // Test node boundaries
        let (_, node, _) = generator.extract.decompose(snowid);
        assert!(node <= 1023);

        // Test sequence boundaries
        let (_, _, sequence) = generator.extract.decompose(snowid);
        assert!(sequence <= 4095);

        // Test custom bit layout boundaries
        let custom_config = SnowIDConfig::builder().node_bits(12).unwrap().epoch(0).build();

        let custom_gen = SnowID::with_config(4095, custom_config).unwrap();
        let snowid = custom_gen.generate();
        let (_, node, sequence) = custom_gen.extract.decompose(snowid);

        assert!(node <= 4095);
        assert!(sequence <= 1023);
    }

    #[test]
    fn test_zero_node_id() {
        let generator = SnowID::new(0).unwrap();
        let snowid = generator.generate();
        let (_, node, _) = generator.extract.decompose(snowid);
        assert_eq!(node, 0);
    }

    #[test]
    fn test_component_max_values() {
        let generator = SnowID::new(1023).unwrap();
        let snowid = generator.generate();
        let (timestamp, node, sequence) = generator.extract.decompose(snowid);

        assert!(timestamp <= generator.config.timestamp_mask());
        assert!(node <= generator.config.max_node_id());
        assert!(sequence <= generator.config.max_sequence_id());
    }

    #[test]
    fn test_future_epoch_is_rejected() {
        let future_epoch = generator_now_unix_ms() + 60_000;
        let config = SnowIDConfig::builder().epoch(future_epoch).build();

        assert!(matches!(SnowID::with_config(1, config), Err(SnowIDError::InvalidEpoch { .. })));
    }

    #[test]
    fn test_epoch_beyond_timestamp_horizon_is_rejected() {
        let old_epoch = 0;
        let config = SnowIDConfig::builder().epoch(old_epoch).build();

        if generator_now_unix_ms() > SnowID::MAX_TIMESTAMP {
            assert!(matches!(SnowID::with_config(1, config), Err(SnowIDError::InvalidEpoch { .. })));
        }
    }

    fn generator_now_unix_ms() -> u64 {
        wall_clock_ms(0)
    }
}
