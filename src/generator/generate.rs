//! ID generation logic
//!
//! Core generate() implementation with fast and slow paths

use std::sync::atomic::Ordering;

use super::SnowID;
use super::state::State;
use super::wait::next_backoff;

struct LogicalBatchStart {
    base_ts: u64,
    start: usize,
    capacity: usize,
}

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
        self.generate_unbounded()
    }

    /// Generate a new SnowID that waits for wall-clock time when sequence values are exhausted.
    #[inline(always)]
    pub fn generate_strict(&self) -> u64 {
        self.generate_strict_loop()
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
            if now >= current.timestamp()
                && let Some(id) = self.try_generate_once(now, current)
            {
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
            if now >= current.timestamp()
                && let Some(written) = self.try_reserve_batch(now, current, out)
            {
                return written;
            }

            let timestamp = current.timestamp();
            if now <= timestamp && current.sequence() >= self.max_seq {
                return 0;
            }
        }
    }

    /// Generate a new SnowID without waiting for wall-clock time on sequence exhaustion.
    ///
    /// When the current millisecond has no remaining sequence values, this method advances the
    /// generator's logical timestamp and returns immediately. The timestamp component can run
    /// ahead of wall-clock time under sustained overload.
    #[inline]
    pub fn generate_unbounded(&self) -> u64 {
        loop {
            let now = self.now_ms();
            let current = State::from_raw(self.state.load(Ordering::Acquire));
            if let Some(id) = self.try_generate_unbounded_once(now, current) {
                return id;
            }
        }
    }

    /// Fill `out` with SnowIDs without waiting for wall-clock time on sequence exhaustion.
    ///
    /// This reserves a logical timestamp range with one atomic state update, then fills the full
    /// buffer. The timestamp component can run ahead of wall-clock time under sustained overload.
    #[inline]
    pub fn generate_batch(&self, out: &mut [u64]) {
        if out.is_empty() {
            return;
        }

        loop {
            let now = self.now_ms();
            let current = State::from_raw(self.state.load(Ordering::Acquire));
            if self.try_reserve_logical_batch(now, current, out) {
                return;
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
    fn try_generate_unbounded_once(&self, now: u64, current: State) -> Option<u64> {
        let ts = current.timestamp();
        let seq = current.sequence();
        let (new_ts, new_seq) = if now > ts {
            (now, 0)
        } else if seq < self.max_seq {
            (ts, seq + 1)
        } else {
            (ts + 1, 0)
        };

        self.cas_state(current, State::new(new_ts, new_seq)).then(|| self.assemble_id(new_ts, new_seq))
    }

    #[inline(always)]
    fn try_reserve_batch(&self, now: u64, current: State, out: &mut [u64]) -> Option<usize> {
        let ts = current.timestamp();
        let start_seq = self.next_strict_sequence(now, current)?;

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

    #[inline]
    fn try_reserve_logical_batch(&self, now: u64, current: State, out: &mut [u64]) -> bool {
        let ts = current.timestamp();
        let base_ts = now.max(ts);
        let capacity = usize::from(self.max_seq) + 1;
        let start = self.next_logical_sequence_index(base_ts, current);
        let final_index = start + out.len() - 1;
        let final_ts = base_ts.saturating_add((final_index / capacity) as u64);
        let final_seq = u16::try_from(final_index % capacity).unwrap_or(self.max_seq);

        if !self.cas_state(current, State::new(final_ts, final_seq)) {
            return false;
        }

        self.fill_logical_batch(out, LogicalBatchStart { base_ts, start, capacity });
        true
    }

    #[inline(always)]
    fn next_strict_sequence(&self, now: u64, current: State) -> Option<u16> {
        if now > current.timestamp() {
            return Some(0);
        }
        let seq = current.sequence();
        (seq < self.max_seq).then(|| seq + 1)
    }

    #[inline(always)]
    fn next_logical_sequence_index(&self, base_ts: u64, current: State) -> usize {
        if base_ts > current.timestamp() {
            return 0;
        }
        let seq = current.sequence();
        if seq < self.max_seq { usize::from(seq) + 1 } else { usize::from(self.max_seq) + 1 }
    }

    #[inline]
    fn fill_logical_batch(&self, out: &mut [u64], batch: LogicalBatchStart) {
        for (offset, id) in out.iter_mut().enumerate() {
            let index = batch.start + offset;
            let timestamp = batch.base_ts + (index / batch.capacity) as u64;
            let sequence = u16::try_from(index % batch.capacity).unwrap_or(self.max_seq);
            *id = self.assemble_id(timestamp, sequence);
        }
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

    #[cold]
    #[inline(never)]
    fn generate_strict_loop(&self) -> u64 {
        let mut backoff_ms = 1u64;

        loop {
            let now = self.now_ms();
            let current = State::from_raw(self.state.load(Ordering::Acquire));
            let timestamp = current.timestamp();

            if now >= timestamp
                && let Some(id) = self.try_generate_once(now, current)
            {
                return id;
            }

            if now >= timestamp && (now > timestamp || current.sequence() < self.max_seq) {
                continue;
            }

            self.wait_next_millis(timestamp, backoff_ms);
            backoff_ms = next_backoff(backoff_ms);
        }
    }
}
