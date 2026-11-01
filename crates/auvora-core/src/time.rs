//! Wall-clock access.
//!
//! Code MUST NOT call `SystemTime::now()` directly (enforced by clippy's
//! `disallowed-methods`, ADR-0047 §10). It receives a [`Clock`] instead, so
//! expiry, rotation and replay logic stays deterministic under test.

use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// Source of the current time.
pub trait Clock: Send + Sync {
    /// Current wall-clock time.
    fn now(&self) -> SystemTime;

    /// Milliseconds since the Unix epoch, saturating at 0 for pre-epoch clocks.
    fn unix_millis(&self) -> u64 {
        self.now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |d| u64::try_from(d.as_millis()).unwrap_or(u64::MAX))
    }

    /// Microseconds since the Unix epoch (PostgreSQL `timestamptz` precision),
    /// saturating at 0 for pre-epoch clocks.
    fn unix_micros(&self) -> i64 {
        self.now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |d| i64::try_from(d.as_micros()).unwrap_or(i64::MAX))
    }
}

/// The real system clock. This is the only place allowed to read wall-clock time.
#[derive(Debug, Clone, Copy, Default)]
pub struct SystemClock;

impl Clock for SystemClock {
    #[allow(clippy::disallowed_methods)] // The single sanctioned call site (ADR-0047 §10).
    fn now(&self) -> SystemTime {
        SystemTime::now()
    }
}

/// A clock frozen at a fixed instant. Intended for tests and simulations.
#[derive(Debug, Clone, Copy)]
pub struct FixedClock(SystemTime);

impl FixedClock {
    /// A clock fixed at `millis` milliseconds after the Unix epoch.
    #[must_use]
    pub fn at_unix_millis(millis: u64) -> Self {
        Self(UNIX_EPOCH + Duration::from_millis(millis))
    }
}

impl Clock for FixedClock {
    fn now(&self) -> SystemTime {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fixed_clock_reports_its_instant() {
        let clock = FixedClock::at_unix_millis(1_762_000_000_123);
        assert_eq!(clock.unix_millis(), 1_762_000_000_123);
        assert_eq!(clock.unix_micros(), 1_762_000_000_123_000);
    }

    #[test]
    fn system_clock_is_after_2025() {
        // 2025-01-01T00:00:00Z
        assert!(SystemClock.unix_millis() > 1_735_689_600_000);
    }
}
