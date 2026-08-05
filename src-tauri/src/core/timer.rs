//! Per-subject study timer state machine.
//!
//! TODO: model the idle → running → paused → finished transitions and the
//! accumulated-elapsed bookkeeping, driven by [`super::clock`] rather than by
//! real sleeping.
