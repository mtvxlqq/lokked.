//! What a session is timing: a plain stopwatch, a countdown, or Pomodoro.
//!
//! Split out of `mod.rs` so that file stays focused on the state machine
//! itself; this one is just data plus the small lookups [`Timer`](super::Timer)
//! needs from it.

use chrono::TimeDelta;
use serde::{Deserialize, Serialize};

/// Which part of a Pomodoro cycle a session is currently in.
///
/// [`Mode::CountUp`] and [`Mode::CountDown`] sessions never leave
/// [`Work`](SessionPhase::Work) — they have no breaks to move to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SessionPhase {
    Work,
    Break,
    LongBreak,
}

/// How a session's target time, if any, is defined.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Mode {
    /// Counts up with no target; [`Timer::state_at`](super::Timer::state_at)
    /// reports no `remaining` and never reports `finished`.
    CountUp,
    /// Counts down from `target` to zero, then keeps counting — reaching zero
    /// only flips `finished` in the reported state, it does not stop the
    /// timer. Ending the session is a separate, explicit call.
    CountDown { target: TimeDelta },
    /// Alternates `Work` and `Break`, taking a longer `LongBreak` every
    /// `cycles_before_long_break`th work phase. `auto_start_next` is a flag
    /// the caller reads with [`Mode::auto_start_next`] to decide whether to
    /// call [`Timer::skip_phase`](super::Timer::skip_phase) automatically
    /// once a phase's target is reached, or wait for the user.
    Pomodoro {
        work: TimeDelta,
        short_break: TimeDelta,
        long_break: TimeDelta,
        cycles_before_long_break: u32,
        auto_start_next: bool,
    },
}

impl Mode {
    /// Whether this mode has more than one phase to move between. `CountUp`
    /// and `CountDown` sessions stay in `Work` forever, so
    /// [`Timer::skip_phase`](super::Timer::skip_phase) is rejected for them.
    pub(super) fn has_phases(&self) -> bool {
        matches!(self, Mode::Pomodoro { .. })
    }

    /// Whether a finished phase should advance on its own rather than wait
    /// for the user. Only meaningful for [`Mode::Pomodoro`]; always `false`
    /// otherwise, since `CountUp`/`CountDown` have nothing to advance to.
    pub(super) fn auto_start_next(&self) -> bool {
        match self {
            Mode::Pomodoro {
                auto_start_next, ..
            } => *auto_start_next,
            Mode::CountUp | Mode::CountDown { .. } => false,
        }
    }

    /// The target duration for `phase`, or `None` if that phase has no
    /// target (a plain stopwatch, or a phase this mode never enters — the
    /// latter should not happen since [`Timer`](super::Timer) only ever asks
    /// for its own current phase, but returning `None` rather than panicking
    /// keeps this lookup total).
    pub(super) fn target(&self, phase: SessionPhase) -> Option<TimeDelta> {
        match (self, phase) {
            (Mode::CountUp, _) => None,
            (Mode::CountDown { target }, SessionPhase::Work) => Some(*target),
            (Mode::CountDown { .. }, _) => None,
            (Mode::Pomodoro { work, .. }, SessionPhase::Work) => Some(*work),
            (Mode::Pomodoro { short_break, .. }, SessionPhase::Break) => Some(*short_break),
            (Mode::Pomodoro { long_break, .. }, SessionPhase::LongBreak) => Some(*long_break),
        }
    }
}

/// A snapshot of a [`Timer`](super::Timer) at a point in time — what the UI
/// polls to render the countdown.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct TimerState {
    /// Time accrued in the current phase.
    pub elapsed: TimeDelta,
    /// Time left in the current phase, or `None` for `CountUp`, which has no
    /// target.
    pub remaining: Option<TimeDelta>,
    pub phase: SessionPhase,
    /// 1-based position within the current group of
    /// `cycles_before_long_break` work phases. Always `1` outside
    /// `Mode::Pomodoro`.
    pub cycle: u32,
    /// Whether the current phase's target has been reached. Always `false`
    /// for `CountUp`, which has no target to reach.
    pub finished: bool,
}
