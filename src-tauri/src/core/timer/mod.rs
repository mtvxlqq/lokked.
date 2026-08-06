//! Per-subject study timer state machine.
//!
//! A [`Timer`] is a plain record of *when things happened*: when the current
//! phase started, when each pause opened and closed, when the session
//! finished. It holds no running total, and it never needs to be ticked.
//! Every duration the UI shows is recomputed on demand from those timestamps:
//!
//! ```text
//! elapsed = (end - phase_started_at) - Σ pauses
//! ```
//!
//! where `end` is the finish time for a finished session and "now"
//! otherwise. That is what makes the timer survive the OS suspending or
//! killing the app mid-session — the one thing a counter incremented by a
//! tick loop cannot do. Consequently a timer never has to be running to stay
//! correct, and reading it twice a second or twice a day gives the same
//! answer.
//!
//! The idle state of the doc-level "idle → running → paused → finished"
//! lifecycle is simply the absence of a `Timer`; a timer exists only once it
//! has been started, so the type has no state in which it is not a session.
//!
//! A session is one [`Mode`] chosen at [`Timer::start`] and never changed.
//! `CountUp`/`CountDown` stay in [`SessionPhase::Work`] for their whole life;
//! `Pomodoro` moves between `Work`, `Break` and `LongBreak` via
//! [`Timer::skip_phase`]. Deciding *when* to call `skip_phase` — on a timeout,
//! or on a button tap — is a policy question for the caller, driven by
//! [`Mode::auto_start_next`] and the `finished` flag in [`TimerState`]; this
//! module only tracks *whether* a transition is valid, never triggers one on
//! its own. A phase transition does not retain the phase it left: any caller
//! that needs to persist a finished phase (writing a `sessions` row, say)
//! must read [`Timer::state_at`] and [`Timer::interruptions`] before calling
//! `skip_phase`, since both reset for the new phase.
//!
//! Wall-clock time can move backwards (an NTP correction, a user editing the
//! system clock), so every span computed here is clamped at zero rather than
//! being allowed to go negative.
//!
//! Behavioural tests live in `src-tauri/tests/timer.rs`.

mod mode;

pub use mode::{Mode, SessionPhase, TimerState};

use chrono::{DateTime, TimeDelta, Utc};
use serde::{Deserialize, Serialize};

use super::clock::Clock;

/// A transition that does not exist from the timer's current state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TimerError {
    /// Tried to pause a timer that is already paused.
    AlreadyPaused,
    /// Tried to resume a timer that is not paused.
    NotPaused,
    /// Tried to change a timer that has already finished. Finished sessions
    /// are immutable: start a new timer instead.
    AlreadyFinished,
    /// Tried to skip to the next phase on a mode that has only one phase
    /// (`CountUp` or `CountDown`).
    NoNextPhase,
}

impl std::fmt::Display for TimerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let message = match self {
            Self::AlreadyPaused => "the timer is already paused",
            Self::NotPaused => "the timer is not paused",
            Self::AlreadyFinished => "the timer has already finished",
            Self::NoNextPhase => "this mode has no next phase to skip to",
        };
        f.write_str(message)
    }
}

impl std::error::Error for TimerError {}

/// A closed interval during which the timer was paused, within the current
/// phase — a phase transition discards them, since the phase they belong to
/// no longer exists once it has been left.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Pause {
    /// When the pause began.
    pub started_at: DateTime<Utc>,
    /// When the timer was resumed, or when the session was finished.
    pub ended_at: DateTime<Utc>,
}

impl Pause {
    /// How long the pause lasted, never negative.
    fn duration(&self) -> TimeDelta {
        non_negative(self.ended_at - self.started_at)
    }
}

/// Where a session is in its running/paused/finished lifecycle — orthogonal
/// to [`SessionPhase`], which tracks *which part of a Pomodoro cycle* is
/// active. A session can be paused during a break just as easily as during
/// work.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "status")]
pub enum RunState {
    /// Counting study time.
    Running,
    /// Stopped since the given time; study time is not accruing.
    Paused { since: DateTime<Utc> },
    /// Over. The timer is now immutable and its elapsed time is fixed.
    Finished { at: DateTime<Utc> },
}

/// One study session's timer.
///
/// Cheap to clone, serialisable, and comparable — a timer read back out of
/// storage equals the one that was written.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Timer {
    mode: Mode,
    /// 1-based position within the current group of
    /// `cycles_before_long_break` work phases. Meaningless outside
    /// `Mode::Pomodoro`, where it stays `1`.
    cycle: u32,
    /// Times [`Timer::mark_interruption`] was called during the *current*
    /// phase. Resets to zero on [`Timer::skip_phase`], since each phase is
    /// its own record once persisted.
    interruptions: u32,
    phase: SessionPhase,
    phase_started_at: DateTime<Utc>,
    /// Closed pauses within the current phase. An in-progress pause is not
    /// here; it lives in [`RunState::Paused`] until it closes.
    pauses: Vec<Pause>,
    #[serde(flatten)]
    run: RunState,
}

impl Timer {
    /// Start a session in `mode`, in the `Work` phase, now.
    pub fn start(mode: Mode, clock: &dyn Clock) -> Self {
        Self {
            mode,
            cycle: 1,
            interruptions: 0,
            phase: SessionPhase::Work,
            phase_started_at: clock.now(),
            pauses: Vec::new(),
            run: RunState::Running,
        }
    }

    /// The session's mode. Fixed for the session's whole life.
    pub fn mode(&self) -> Mode {
        self.mode
    }

    /// Which part of the cycle is active right now.
    pub fn phase(&self) -> SessionPhase {
        self.phase
    }

    /// When the current phase started.
    pub fn phase_started_at(&self) -> DateTime<Utc> {
        self.phase_started_at
    }

    /// The running/paused/finished lifecycle state.
    pub fn run_state(&self) -> RunState {
        self.run
    }

    /// Whether study time is currently accruing.
    pub fn is_running(&self) -> bool {
        matches!(self.run, RunState::Running)
    }

    /// Whether the session is paused.
    pub fn is_paused(&self) -> bool {
        matches!(self.run, RunState::Paused { .. })
    }

    /// Whether the session is over.
    pub fn is_finished(&self) -> bool {
        matches!(self.run, RunState::Finished { .. })
    }

    /// When the session finished, if it has.
    pub fn finished_at(&self) -> Option<DateTime<Utc>> {
        match self.run {
            RunState::Finished { at } => Some(at),
            _ => None,
        }
    }

    /// The pauses that have already closed within the current phase, oldest
    /// first.
    pub fn pauses(&self) -> &[Pause] {
        &self.pauses
    }

    /// How many interruptions have been marked during the current phase.
    pub fn interruptions(&self) -> u32 {
        self.interruptions
    }

    /// Whether a finished phase should advance on its own. The caller reads
    /// this alongside [`TimerState::finished`] to decide whether to call
    /// [`Timer::skip_phase`] the moment a phase's target is reached, or to
    /// wait for the user; this module never advances a phase by itself.
    pub fn auto_start_next(&self) -> bool {
        self.mode.auto_start_next()
    }

    /// Stop accruing study time.
    ///
    /// Fails if the timer is already paused or has finished; in either case
    /// the timer is left exactly as it was.
    pub fn pause(&mut self, clock: &dyn Clock) -> Result<(), TimerError> {
        match self.run {
            RunState::Running => {
                self.run = RunState::Paused { since: clock.now() };
                Ok(())
            }
            RunState::Paused { .. } => Err(TimerError::AlreadyPaused),
            RunState::Finished { .. } => Err(TimerError::AlreadyFinished),
        }
    }

    /// Start accruing study time again, closing the open pause.
    ///
    /// Fails if the timer is not paused; the timer is left exactly as it was.
    pub fn resume(&mut self, clock: &dyn Clock) -> Result<(), TimerError> {
        match self.run {
            RunState::Paused { since } => {
                self.close_pause(since, clock.now());
                self.run = RunState::Running;
                Ok(())
            }
            RunState::Running => Err(TimerError::NotPaused),
            RunState::Finished { .. } => Err(TimerError::AlreadyFinished),
        }
    }

    /// End the session, freezing its elapsed time.
    ///
    /// Finishing while paused closes the open pause at the finish time, so a
    /// session never carries a pause that stays open forever.
    pub fn finish(&mut self, clock: &dyn Clock) -> Result<(), TimerError> {
        let now = clock.now();
        match self.run {
            RunState::Running => {
                self.run = RunState::Finished { at: now };
                Ok(())
            }
            RunState::Paused { since } => {
                self.close_pause(since, now);
                self.run = RunState::Finished { at: now };
                Ok(())
            }
            RunState::Finished { .. } => Err(TimerError::AlreadyFinished),
        }
    }

    /// Record that the user got distracted, without stopping the timer.
    ///
    /// Fails only if the session has already finished.
    pub fn mark_interruption(&mut self) -> Result<(), TimerError> {
        if self.is_finished() {
            return Err(TimerError::AlreadyFinished);
        }
        self.interruptions += 1;
        Ok(())
    }

    /// Force-move to the next phase, regardless of how much of the current
    /// one's target has elapsed. Always leaves the new phase `Running`, even
    /// if the phase being left was paused.
    ///
    /// Only valid for [`Mode::Pomodoro`] — `CountUp` and `CountDown` have
    /// only one phase and reject this with [`TimerError::NoNextPhase`].
    /// Also fails if the session has already finished.
    pub fn skip_phase(&mut self, clock: &dyn Clock) -> Result<(), TimerError> {
        if !self.mode.has_phases() {
            return Err(TimerError::NoNextPhase);
        }
        let now = clock.now();
        match self.run {
            RunState::Finished { .. } => return Err(TimerError::AlreadyFinished),
            RunState::Paused { since } => self.close_pause(since, now),
            RunState::Running => {}
        }

        let (next_phase, next_cycle) = match (self.phase, self.mode) {
            (
                SessionPhase::Work,
                Mode::Pomodoro {
                    cycles_before_long_break,
                    ..
                },
            ) => {
                if self.cycle >= cycles_before_long_break {
                    (SessionPhase::LongBreak, self.cycle)
                } else {
                    (SessionPhase::Break, self.cycle)
                }
            }
            (SessionPhase::Break, _) => (SessionPhase::Work, self.cycle + 1),
            (SessionPhase::LongBreak, _) => (SessionPhase::Work, 1),
            // Reachable only via `Mode::Pomodoro`, guarded by `has_phases`
            // above, so `Work` always has cycle data to match on.
            (SessionPhase::Work, _) => unreachable!("non-Pomodoro modes never reach skip_phase"),
        };

        self.phase = next_phase;
        self.cycle = next_cycle;
        self.phase_started_at = now;
        self.pauses.clear();
        self.interruptions = 0;
        self.run = RunState::Running;
        Ok(())
    }

    /// Time accrued in the current phase: wall-clock time since it started,
    /// minus every pause within it.
    ///
    /// Constant once the session has finished, so the `clock` is then unused.
    pub fn elapsed(&self, clock: &dyn Clock) -> TimeDelta {
        let end = self.end(clock);
        non_negative(end - self.phase_started_at - self.paused_until(end))
    }

    /// Time left in the current phase, or `None` if it has no target.
    pub fn remaining(&self, clock: &dyn Clock) -> Option<TimeDelta> {
        let target = self.mode.target(self.phase)?;
        Some(non_negative(target - self.elapsed(clock)))
    }

    /// Total time spent paused in the current phase, including a pause that
    /// is still open.
    pub fn paused(&self, clock: &dyn Clock) -> TimeDelta {
        self.paused_until(self.end(clock))
    }

    /// A snapshot of elapsed/remaining/phase/cycle/finished at `clock`'s
    /// current time — what the UI polls to render the countdown.
    pub fn state_at(&self, clock: &dyn Clock) -> TimerState {
        let elapsed = self.elapsed(clock);
        let remaining = self.remaining(clock);
        TimerState {
            elapsed,
            remaining,
            phase: self.phase,
            cycle: self.cycle,
            finished: matches!(remaining, Some(r) if r <= TimeDelta::zero()),
        }
    }

    /// The instant durations are measured up to: the finish time if the
    /// session has finished, otherwise now.
    fn end(&self, clock: &dyn Clock) -> DateTime<Utc> {
        match self.run {
            RunState::Finished { at } => at,
            _ => clock.now(),
        }
    }

    /// Time paused up to `end`, counting an open pause as running until then.
    fn paused_until(&self, end: DateTime<Utc>) -> TimeDelta {
        let closed: TimeDelta = self.pauses.iter().map(Pause::duration).sum();
        let open = match self.run {
            RunState::Paused { since } => non_negative(end - since),
            _ => TimeDelta::zero(),
        };
        closed + open
    }

    fn close_pause(&mut self, started_at: DateTime<Utc>, ended_at: DateTime<Utc>) {
        self.pauses.push(Pause {
            started_at,
            ended_at,
        });
    }
}

/// Clamp a span at zero, so a clock that jumped backwards cannot make a
/// duration negative.
fn non_negative(delta: TimeDelta) -> TimeDelta {
    delta.max(TimeDelta::zero())
}
