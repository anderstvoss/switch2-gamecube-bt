//! Bounded operation and cancellation primitives.

use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::{Duration, Instant},
};

/// A monotonic deadline for an operation that waits on hardware.
#[derive(Clone, Copy, Debug)]
pub struct Deadline(Instant);

impl Deadline {
    /// Creates a deadline relative to the current monotonic clock.
    #[must_use]
    pub fn after(duration: Duration) -> Self {
        Self(Instant::now() + duration)
    }

    /// Returns whether the deadline has elapsed.
    #[must_use]
    pub fn has_elapsed(self) -> bool {
        Instant::now() >= self.0
    }

    /// Returns the remaining bounded duration.
    #[must_use]
    pub fn remaining(self) -> Duration {
        self.0.saturating_duration_since(Instant::now())
    }
}

/// A clonable cooperative cancellation signal.
#[derive(Clone, Debug, Default)]
pub struct CancellationToken(Arc<AtomicBool>);

impl CancellationToken {
    /// Requests cancellation.
    pub fn cancel(&self) {
        self.0.store(true, Ordering::Release);
    }

    /// Returns whether cancellation was requested.
    #[must_use]
    pub fn is_cancelled(&self) -> bool {
        self.0.load(Ordering::Acquire)
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::{CancellationToken, Deadline};

    #[test]
    fn cancellation_is_shared_between_clones() {
        let first = CancellationToken::default();
        let second = first.clone();
        first.cancel();
        assert!(second.is_cancelled());
    }

    #[test]
    fn deadlines_are_bounded() {
        let deadline = Deadline::after(Duration::ZERO);
        assert!(deadline.has_elapsed());
        assert_eq!(deadline.remaining(), Duration::ZERO);
    }
}
