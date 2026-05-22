//! ID generation logic
//!
//! Core generate() implementation with fast and slow paths

use std::sync::atomic::Ordering;

use super::SnowID;
use super::state::State;
use super::wait::next_backoff;

/// Error returned by non-blocking generation APIs when the current millisecond is exhausted.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TryGenerateError {
    /// Timestamp whose sequence space is exhausted.
    pub timestamp: u64,
}

impl SnowID {
    /// Generate a new SnowID
    #[inline(always)]
    pub fn generate(&self) -> u64 {
        let now = self.now_ms();
        let current = State::from_raw(self.state.load(Ordering::Acquire));

        // Fast path: try to claim or increment
        if let Some(id) = self.try_generate_once(now, current) {
            return id;
        }

        self.generate_slow_path()
    }

    /// Try to generate a new SnowID without waiting for the next millisecond.
    ///
    /// This returns an error instead of spinning or sleeping when the current
    /// millisecond has no remaining sequence values.
    #[inline]
    pub fn try_generate(&self) -> Result<u64, TryGenerateError> {
        loop {
            let now = self.now_ms();
            let current = State::from_raw(self.state.load(Ordering::Acquire));
            if let Some(id) = self.try_generate_once(now, current) {
                return Ok(id);
            }

            let timestamp = current.timestamp();
            if now <= timestamp && current.sequence() >= self.max_seq {
                return Err(TryGenerateError { timestamp });
            }
        }
    }

    /// Generate as many IDs as can be reserved immediately into `out`.
    ///
    /// Returns the number of IDs written. This method never waits for the next
    /// millisecond; callers can retry later when fewer than `out.len()` IDs are
    /// returned.
    #[inline]
    pub fn try_generate_batch(&self, out: &mut [u64]) -> usize {
        if out.is_empty() {
            return 0;
        }

        loop {
            let now = self.now_ms();
            let current = State::from_raw(self.state.load(Ordering::Acquire));
            if let Some(written) = self.try_reserve_batch(now, current, out) {
                return written;
            }

            let timestamp = current.timestamp();
            if now <= timestamp && current.sequence() >= self.max_seq {
                return 0;
            }
        }
    }

    /// Attempt a single generation: claim new ms or increment sequence.
    #[inline(always)]
    fn try_generate_once(&self, now: u64, current: State) -> Option<u64> {
        let ts = current.timestamp();
        if now > ts {
            let new_state = State::new(now, 0);
            if self.cas_state(current, new_state) {
                return Some(self.assemble_id(now, 0));
            }
        } else {
            let seq = current.sequence();
            if seq < self.max_seq && self.cas_state(current, State::from_raw(current.raw() + 1)) {
                return Some(self.assemble_id(ts, seq + 1));
            }
        }
        None
    }

    #[inline(always)]
    fn try_reserve_batch(&self, now: u64, current: State, out: &mut [u64]) -> Option<usize> {
        let ts = current.timestamp();
        let start_seq = if now > ts { 0 } else { current.sequence().saturating_add(1) };
        if start_seq > self.max_seq {
            return None;
        }

        let remaining = self.max_seq - start_seq;
        let requested = u16::try_from(out.len().saturating_sub(1)).unwrap_or(u16::MAX);
        let last_offset = requested.min(remaining);
        let written = usize::from(last_offset) + 1;
        let last_seq = start_seq + last_offset;
        let new_ts = if now > ts { now } else { ts };
        let new_state = State::new(new_ts, last_seq);

        if !self.cas_state(current, new_state) {
            return None;
        }

        for (offset, id) in (0..=last_offset).zip(out.iter_mut()) {
            *id = self.assemble_id(new_ts, start_seq + offset);
        }
        Some(written)
    }

    /// Try to claim new millisecond with sequence 0
    #[inline(always)]
    #[allow(dead_code)]
    pub(crate) fn try_claim_millisecond(&self, current: State, new_ts: u64) -> Option<u64> {
        let new_state = State::new(new_ts, 0);
        self.cas_state(current, new_state).then(|| self.assemble_id(new_ts, 0))
    }

    /// Try to increment sequence within current millisecond
    #[inline(always)]
    #[allow(dead_code)]
    pub(crate) fn try_increment_sequence(&self, current: State) -> Option<u64> {
        let seq = current.sequence();
        if seq < self.max_seq {
            let new_state = State::from_raw(current.raw() + 1);
            self.cas_state(current, new_state).then(|| self.assemble_id(current.timestamp(), seq + 1))
        } else {
            None
        }
    }

    /// Atomic compare-and-swap on state
    #[inline(always)]
    pub(crate) fn cas_state(&self, expected: State, new: State) -> bool {
        self.state
            .compare_exchange_weak(expected.raw(), new.raw(), Ordering::AcqRel, Ordering::Acquire)
            .is_ok()
    }

    /// Slow path for contended generation
    #[cold]
    #[inline(never)]
    fn generate_slow_path(&self) -> u64 {
        let mut backoff_ms = 1u64;

        loop {
            let now = self.now_ms();
            let current = State::from_raw(self.state.load(Ordering::Acquire));
            let timestamp = current.timestamp();

            if let Some(id) = self.try_generate_once(now, current) {
                return id;
            }

            // Retry immediately on CAS contention; only wait when the loaded
            // state shows the current millisecond is actually exhausted.
            if now > timestamp || current.sequence() < self.max_seq {
                continue;
            }

            self.wait_next_millis(timestamp, backoff_ms);
            backoff_ms = next_backoff(backoff_ms);
        }
    }
}
