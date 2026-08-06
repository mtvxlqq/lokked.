//! The real system clock.
//!
//! Lives here rather than next to the [`Clock`] trait because reading the wall
//! clock is a service of the host OS, and [`crate::core`] is forbidden both
//! from I/O and from calling [`Utc::now`] directly. `core` defines what a clock
//! *is*; this module is the one place that actually asks the operating system
//! what time it is.
//!
//! Unlike the other backends in [`crate::platform`], this one needs no
//! per-target implementation — `chrono` already abstracts over the platform —
//! so there is a single type for every target.

use chrono::{DateTime, Utc};

use crate::core::clock::Clock;

/// Reads the current time from the operating system.
#[derive(Debug, Clone, Copy, Default)]
pub struct SystemClock;

impl Clock for SystemClock {
    fn now(&self) -> DateTime<Utc> {
        Utc::now()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_system_clock_does_not_run_backwards() {
        let clock = SystemClock;

        let first = clock.now();
        let second = clock.now();

        assert!(second >= first);
    }

    #[test]
    fn the_system_clock_is_usable_as_a_clock_trait_object() {
        let clock = SystemClock;
        let as_trait: &dyn Clock = &clock;

        // A sanity bound, not a precise assertion: any plausible run of this
        // test happens between 2026 and the end of the century.
        let now = as_trait.now();

        assert!(now.timestamp() > 1_767_225_600); // 2026-01-01
        assert!(now.timestamp() < 4_102_444_800); // 2100-01-01
    }
}
