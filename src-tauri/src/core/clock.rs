//! Time source abstraction.
//!
//! Wraps "what time is it now" behind a trait so timers and the scheduler can
//! be tested deterministically with a fake clock instead of sleeping.
//!
//! TODO: define `trait Clock { fn now(&self) -> Instant; }`, a system-clock
//! implementation and a controllable test clock.
