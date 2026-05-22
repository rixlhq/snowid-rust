//! Sequence rollover and uniqueness tests

#[cfg(test)]
mod tests {
    use crate::tests::test_utils::{assert_ids_monotonic, assert_unique_ids};
    use crate::*;
    use std::collections::HashSet;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_sequence_rollover() {
        let generator = SnowID::new(1).unwrap();
        let initial_id = generator.generate();
        let initial_ts = generator.extract.timestamp(initial_id);
        let mut max_seq = 0;

        for i in 0..10000 {
            let id = generator.generate();
            let (ts, _, seq) = generator.extract.decompose(id);
            max_seq = max_seq.max(seq);

            if ts == initial_ts && seq < max_seq {
                assert!(max_seq <= generator.config.max_sequence_id());
                return;
            } else if ts != initial_ts && i > 0 {
                assert!(seq <= 1, "Sequence should reset on ts change");
                return;
            }

            if i % 100 == 0 {
                thread::yield_now();
            }
        }
        panic!("Sequence did not rollover in 10000 iterations");
    }

    #[test]
    fn test_sequence_overflow_handling() {
        let generator = SnowID::new(1).unwrap();
        let mut last_ts = 0;

        for _ in 0..100000 {
            let id = generator.generate();
            let (ts, _, seq) = generator.extract.decompose(id);

            if ts == last_ts && last_ts > 0 && seq >= generator.config.max_sequence_id() {
                let next = generator.generate();
                let (next_ts, _, next_seq) = generator.extract.decompose(next);
                assert!(next_ts > ts, "Timestamp should advance on overflow");
                assert_eq!(next_seq, 0, "Sequence should reset");
                return;
            }

            last_ts = ts;
            if seq % 1000 == 0 {
                thread::yield_now();
            }
        }
        panic!("Sequence overflow not handled");
    }

    #[test]
    fn test_sequence_monotonicity() {
        let generator = SnowID::new(1).unwrap();
        let ids: Vec<u64> = (0..1000).map(|_| generator.generate()).collect();
        assert_ids_monotonic(&ids);
    }

    #[test]
    fn test_10k_unique_ids() {
        const COUNT: usize = 10_000;
        let generator = SnowID::new(1).unwrap();
        let ids: Vec<u64> = (0..COUNT).map(|_| generator.generate()).collect();

        assert_unique_ids(&ids, COUNT);
        assert_ids_monotonic(&ids);

        // Analyze timestamp distribution
        let timestamps: HashSet<_> = ids.iter().map(|id| generator.extract.timestamp(*id)).collect();
        assert!(!timestamps.is_empty(), "Should have at least one timestamp");
    }

    #[test]
    fn test_try_generate_matches_generate_contract() {
        let generator = SnowID::new(1).unwrap();
        let ids: Vec<u64> = (0..1000).map(|_| generator.try_generate().unwrap()).collect();

        assert_unique_ids(&ids, ids.len());
        assert_ids_monotonic(&ids);
    }

    #[test]
    fn test_try_generate_batch_fills_unique_monotonic_ids() {
        let generator = SnowID::new(1).unwrap();
        let mut ids = [0u64; 128];

        let written = generator.try_generate_batch(&mut ids);

        assert_eq!(written, ids.len());
        assert_unique_ids(&ids, ids.len());
        assert_ids_monotonic(&ids);
    }

    #[test]
    fn test_try_generate_batch_returns_partial_capacity() {
        let config = SnowIDConfig::builder().node_bits(16).unwrap().enable_spin(false).build();
        let generator = SnowID::with_config(1, config).unwrap();
        let mut ids = [0u64; 128];

        let written = generator.try_generate_batch(&mut ids);

        assert_eq!(written, usize::from(generator.config.max_sequence_id()) + 1);
        assert_unique_ids(&ids[..written], written);
        assert_ids_monotonic(&ids[..written]);
    }

    #[test]
    fn test_generate_unbounded_advances_logical_timestamp() {
        let config = SnowIDConfig::builder().node_bits(16).unwrap().enable_spin(false).build();
        let generator = SnowID::with_config(1, config).unwrap();
        let ids: Vec<u64> = (0..128).map(|_| generator.generate_unbounded()).collect();

        assert_unique_ids(&ids, ids.len());
        assert_ids_monotonic(&ids);
        assert!(generator.extract.timestamp(ids[64]) > generator.extract.timestamp(ids[0]));
        assert_eq!(generator.extract.sequence(ids[64]), 0);
    }

    #[test]
    fn test_generate_batch_fills_full_buffer_across_logical_timestamps() {
        let config = SnowIDConfig::builder().node_bits(16).unwrap().enable_spin(false).build();
        let generator = SnowID::with_config(1, config).unwrap();
        let mut ids = [0u64; 128];

        generator.generate_batch(&mut ids);

        assert_unique_ids(&ids, ids.len());
        assert_ids_monotonic(&ids);
        assert!(generator.extract.timestamp(ids[64]) > generator.extract.timestamp(ids[0]));
        assert_eq!(generator.extract.sequence(ids[64]), 0);
    }

    #[test]
    fn test_generate_defaults_to_logical_timestamp_overflow() {
        let config = SnowIDConfig::builder().node_bits(16).unwrap().enable_spin(false).build();
        let generator = SnowID::with_config(1, config).unwrap();
        let ids: Vec<u64> = (0..128).map(|_| generator.generate()).collect();

        assert_unique_ids(&ids, ids.len());
        assert_ids_monotonic(&ids);
        assert!(generator.extract.timestamp(ids[64]) > generator.extract.timestamp(ids[0]));
        assert_eq!(generator.extract.sequence(ids[64]), 0);
    }

    #[test]
    fn test_generate_batch_handles_sustained_logical_overflow() {
        let config = SnowIDConfig::builder().node_bits(16).unwrap().enable_spin(false).build();
        let generator = SnowID::with_config(1, config).unwrap();
        let mut ids = [0u64; 4096];

        generator.generate_batch(&mut ids);

        assert_unique_ids(&ids, ids.len());
        assert_ids_monotonic(&ids);
        assert_eq!(generator.extract.sequence(ids[0]), 0);
        assert_eq!(generator.extract.sequence(ids[64]), 0);
        assert_eq!(generator.extract.sequence(ids[4095]), 63);
        assert_eq!(generator.extract.timestamp(ids[4095]) - generator.extract.timestamp(ids[0]), 63);
    }

    #[test]
    fn test_logical_future_state_never_moves_backwards() {
        let config = SnowIDConfig::builder().node_bits(16).unwrap().enable_spin(false).build();
        let generator = SnowID::with_config(1, config).unwrap();
        let mut ids = [0u64; 4096];
        generator.generate_batch(&mut ids);

        let next = generator.generate();

        assert!(next > ids[4095]);
        assert!(generator.extract.timestamp(next) >= generator.extract.timestamp(ids[4095]));
    }

    #[test]
    fn test_try_generate_returns_err_when_logical_state_is_future() {
        let config = SnowIDConfig::builder().node_bits(16).unwrap().enable_spin(false).build();
        let generator = SnowID::with_config(1, config).unwrap();
        let mut ids = [0u64; 128];
        generator.generate_batch(&mut ids);

        let err = generator.try_generate().unwrap_err();

        assert_eq!(err.timestamp, generator.extract.timestamp(ids[127]));
    }

    #[test]
    fn test_try_generate_batch_returns_zero_when_logical_state_is_future() {
        let config = SnowIDConfig::builder().node_bits(16).unwrap().enable_spin(false).build();
        let generator = SnowID::with_config(1, config).unwrap();
        let mut ids = [0u64; 128];
        let mut out = [0u64; 4];
        generator.generate_batch(&mut ids);

        assert_eq!(generator.try_generate_batch(&mut out), 0);
    }

    #[test]
    fn test_strict_generation_waits_until_logical_future_catches_up() {
        let config = SnowIDConfig::builder().node_bits(16).unwrap().enable_spin(false).build();
        let generator = SnowID::with_config(1, config).unwrap();
        let mut ids = [0u64; 128];
        generator.generate_batch(&mut ids);
        let future_ts = generator.extract.timestamp(ids[127]);

        thread::sleep(Duration::from_millis(3));
        let strict = generator.generate_strict();

        assert!(generator.extract.timestamp(strict) >= future_ts);
        assert!(strict > ids[127]);
    }

    #[test]
    fn test_try_generate_batch_does_not_duplicate_u16_max_sequence() {
        let config = SnowIDConfig::builder().node_bits(6).unwrap().build();
        let generator = SnowID::with_config(1, config).unwrap();
        let mut first = vec![0u64; 65_536 * 32];
        let mut second = [0u64; 1];

        assert_eq!(generator.try_generate_batch(&mut first), 65_536);
        assert_eq!(generator.extract.sequence(first[65_535]), u16::MAX);

        if generator.try_generate_batch(&mut second) == 1 {
            assert!(second[0] > first[65_535]);
        }
    }

    #[test]
    fn test_generate_batch_crosses_u16_max_sequence_without_duplicates() {
        let config = SnowIDConfig::builder().node_bits(6).unwrap().build();
        let generator = SnowID::with_config(1, config).unwrap();
        let mut ids = vec![0u64; 65_537];

        generator.generate_batch(&mut ids);

        assert_unique_ids(&ids, ids.len());
        assert_ids_monotonic(&ids);
        assert_eq!(generator.extract.sequence(ids[65_535]), u16::MAX);
        assert_eq!(generator.extract.sequence(ids[65_536]), 0);
        assert!(generator.extract.timestamp(ids[65_536]) > generator.extract.timestamp(ids[0]));
    }
}
