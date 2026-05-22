#[cfg(test)]
mod tests {
    use crate::tests::test_utils::{assert_ids_monotonic, assert_unique_ids, wall_clock_ms};
    use crate::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn generate_batch_preserves_uniqueness_and_layout(node_bits in 6u8..=16, batch_len in 1usize..=8192) {
            let config = SnowIDConfig::builder().node_bits(node_bits).unwrap().enable_spin(false).build();
            let generator = SnowID::with_config(1, config).unwrap();
            let mut ids = vec![0u64; batch_len];

            generator.generate_batch(&mut ids);

            assert_unique_ids(&ids, ids.len());
            assert_ids_monotonic(&ids);
            for id in ids {
                let (timestamp, node, sequence) = generator.extract.decompose(id);
                prop_assert!(timestamp <= SnowID::MAX_TIMESTAMP);
                prop_assert_eq!(node, 1);
                prop_assert!(sequence <= generator.config.max_sequence_id());
            }
        }

        #[test]
        fn generate_is_monotonic_under_varied_capacity(node_bits in 6u8..=16, count in 1usize..=20_000) {
            let config = SnowIDConfig::builder().node_bits(node_bits).unwrap().enable_spin(false).build();
            let generator = SnowID::with_config(1, config).unwrap();
            let ids: Vec<u64> = (0..count).map(|_| generator.generate()).collect();

            assert_unique_ids(&ids, ids.len());
            assert_ids_monotonic(&ids);
        }

        #[test]
        fn try_generate_batch_never_exceeds_requested_or_capacity(node_bits in 6u8..=16, batch_len in 1usize..=131_072) {
            let config = SnowIDConfig::builder().node_bits(node_bits).unwrap().enable_spin(false).build();
            let generator = SnowID::with_config(1, config).unwrap();
            let mut ids = vec![0u64; batch_len];

            let written = generator.try_generate_batch(&mut ids);

            prop_assert!(written <= batch_len);
            prop_assert!(written <= usize::from(generator.config.max_sequence_id()) + 1);
            if written > 0 {
                assert_unique_ids(&ids[..written], written);
                assert_ids_monotonic(&ids[..written]);
            }
        }

        #[test]
        fn recent_custom_epochs_preserve_component_bounds(node_bits in 6u8..=16, epoch_offset_ms in 0u64..=86_400_000) {
            let epoch = wall_clock_ms(0) - epoch_offset_ms;
            let config = SnowIDConfig::builder().node_bits(node_bits).unwrap().epoch(epoch).build();
            let generator = SnowID::with_config(1, config).unwrap();
            let id = generator.generate();
            let (timestamp, node, sequence) = generator.extract.decompose(id);

            prop_assert!(timestamp <= epoch_offset_ms + 10);
            prop_assert_eq!(node, 1);
            prop_assert!(sequence <= generator.config.max_sequence_id());
        }
    }
}
