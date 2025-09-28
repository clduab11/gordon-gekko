use std::sync::atomic::{AtomicU64, Ordering};

static GLOBAL_SEQUENCE: AtomicU64 = AtomicU64::new(1);

/// Returns a globally unique, monotonically increasing sequence number.
#[inline]
pub(crate) fn next_sequence() -> u64 {
    GLOBAL_SEQUENCE.fetch_add(1, Ordering::Relaxed)
}
