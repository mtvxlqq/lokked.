//! Time source abstraction.
//!
//! Wraps "what time is it now" behind a trait so timers and the scheduler can
//! be tested deterministically with a fake clock instead of sleeping.
//!
//! The clock deals in **wall-clock UTC**, not in [`std::time::Instant`]. A
//! monotonic instant cannot survive the app being written to the database,
//! killed by the OS and restarted — and on mobile that happens routinely — so
//! every duration in Lokked is the difference between two stored UTC
//! timestamps.
//!
//! Nothing inside [`crate::core`] may call `Utc::now` directly; it goes
//! through a `&dyn Clock` argument so tests can drive time by hand. Asking the
//! OS for the real time is I/O, so the production implementation lives outside
//! this module, in [`crate::platform::clock::SystemClock`]. Only the trait and
//! the test double are here.

use std::sync::Mutex;

use chrono::{DateTime, TimeDelta, Utc};

/// A source of the current wall-clock time in UTC.
///
/// `Send + Sync` because a single clock is shared by the whole app, including
/// Tauri's managed state and any background task.
pub trait Clock: Send + Sync {
    /// The current time, in UTC.
    fn now(&self) -> DateTime<Utc>;
}

/// A clock that only moves when a test tells it to.
///
/// Intended for tests, but deliberately not `#[cfg(test)]`: integration tests
/// under `src-tauri/tests/` are separate crates and need it too.
///
/// ```
/// use chrono::{TimeDelta, TimeZone, Utc};
/// use lokked_lib::core::clock::{Clock, FakeClock};
///
/// let clock = FakeClock::new(Utc.with_ymd_and_hms(2026, 8, 6, 9, 0, 0).unwrap());
/// let start = clock.now();
/// clock.advance(TimeDelta::minutes(25));
/// assert_eq!(clock.now() - start, TimeDelta::minutes(25));
/// ```
#[derive(Debug)]
pub struct FakeClock {
    now: Mutex<DateTime<Utc>>,
}

impl FakeClock {
    /// A fake clock stopped at `now`.
    pub fn new(now: DateTime<Utc>) -> Self {
        Self {
            now: Mutex::new(now),
        }
    }

    /// Move the clock forward (or, with a negative delta, backward) by `delta`.
    ///
    /// Takes `&self` so a test can advance a clock it has already handed out
    /// as a `&dyn Clock`.
    pub fn advance(&self, delta: TimeDelta) {
        let mut now = self.now.lock().expect("FakeClock mutex poisoned");
        *now += delta;
    }

    /// Jump the clock to an arbitrary time, as an NTP correction would.
    pub fn set(&self, now: DateTime<Utc>) {
        *self.now.lock().expect("FakeClock mutex poisoned") = now;
    }
}

impl Clock for FakeClock {
    fn now(&self) -> DateTime<Utc> {
        *self.now.lock().expect("FakeClock mutex poisoned")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn at(hour: u32, minute: u32) -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 8, 6, hour, minute, 0).unwrap()
    }

    #[test]
    fn fake_clock_stands_still_until_advanced() {
        let clock = FakeClock::new(at(9, 0));

        assert_eq!(clock.now(), at(9, 0));
        assert_eq!(clock.now(), at(9, 0));
    }

    #[test]
    fn fake_clock_advances_by_the_given_delta() {
        let clock = FakeClock::new(at(9, 0));

        clock.advance(TimeDelta::minutes(30));

        assert_eq!(clock.now(), at(9, 30));
    }

    #[test]
    fn fake_clock_advances_backwards_on_a_negative_delta() {
        let clock = FakeClock::new(at(9, 0));

        clock.advance(TimeDelta::minutes(-15));

        assert_eq!(clock.now(), at(8, 45));
    }

    #[test]
    fn fake_clock_jumps_to_an_arbitrary_time() {
        let clock = FakeClock::new(at(9, 0));

        clock.set(at(14, 5));

        assert_eq!(clock.now(), at(14, 5));
    }

    #[test]
    fn fake_clock_is_usable_through_the_trait_object() {
        let clock = FakeClock::new(at(9, 0));
        let as_trait: &dyn Clock = &clock;

        clock.advance(TimeDelta::hours(1));

        assert_eq!(as_trait.now(), at(10, 0));
    }
}
