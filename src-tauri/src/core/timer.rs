//! Per-subject study timer state machine.
//!
//! A [`Timer`] is a plain record of *when things happened*: when the session
//! started, when each pause opened and closed, when it finished. It holds no
//! running total, and it never needs to be ticked. Every duration the UI shows
//! is recomputed on demand from those timestamps:
//!
//! ```text
//! elapsed = (end - started_at) - Σ pauses
//! ```
//!
//! where `end` is the finish time for a finished timer and "now" otherwise.
//! That is what makes the timer survive the OS suspending or killing the app
//! mid-session — the one thing a counter incremented by a tick loop cannot do.
//! Consequently a timer never has to be running to stay correct, and reading
//! it twice a second or twice a day gives the same answer.
//!
//! The idle state of the doc-level "idle → running → paused → finished"
//! lifecycle is simply the absence of a `Timer`; a timer exists only once it
//! has been started, so the type has no state in which it is not a session.
//!
//! Wall-clock time can move backwards (an NTP correction, a user editing the
//! system clock), so every span computed here is clamped at zero rather than
//! being allowed to go negative.
//!
//! Behavioural tests live in `src-tauri/tests/timer.rs`.

use chrono::{DateTime, TimeDelta, Utc};
use serde::{Deserialize, Serialize};

use super::clock::Clock;

/// A transition that does not exist from the timer's current phase.
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
}

impl std::fmt::Display for TimerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let message = match self {
            Self::AlreadyPaused => "the timer is already paused",
            Self::NotPaused => "the timer is not paused",
            Self::AlreadyFinished => "the timer has already finished",
        };
        f.write_str(message)
    }
}

impl std::error::Error for TimerError {}

/// A closed interval during which the timer was paused.
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

/// Where a timer is in its lifecycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "phase")]
pub enum Phase {
    /// Counting study time.
    Running,
    /// Stopped since the given time; study time is not accruing.
    Paused { since: DateTime<Utc> },
    /// Over. The timer is now immutable and its elapsed time is fixed.
    Finished { at: DateTime<Utc> },
}

/// One study session's timer.
///
/// Cheap to clone, serialisable, and comparable — a timer read back out of the
/// database equals the one that was written.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Timer {
    started_at: DateTime<Utc>,
    /// Closed pauses, in the order they happened. An in-progress pause is not
    /// here; it lives in [`Phase::Paused`] until it closes.
    pauses: Vec<Pause>,
    #[serde(flatten)]
    phase: Phase,
}

impl Timer {
    /// Start a session now.
    pub fn start(clock: &dyn Clock) -> Self {
        Self {
            started_at: clock.now(),
            pauses: Vec::new(),
            phase: Phase::Running,
        }
    }

    /// When the session started.
    pub fn started_at(&self) -> DateTime<Utc> {
        self.started_at
    }

    /// The current phase.
    pub fn phase(&self) -> Phase {
        self.phase
    }

    /// Whether study time is currently accruing.
    pub fn is_running(&self) -> bool {
        matches!(self.phase, Phase::Running)
    }

    /// Whether the session is paused.
    pub fn is_paused(&self) -> bool {
        matches!(self.phase, Phase::Paused { .. })
    }

    /// Whether the session is over.
    pub fn is_finished(&self) -> bool {
        matches!(self.phase, Phase::Finished { .. })
    }

    /// When the session finished, if it has.
    pub fn finished_at(&self) -> Option<DateTime<Utc>> {
        match self.phase {
            Phase::Finished { at } => Some(at),
            _ => None,
        }
    }

    /// The pauses that have already closed, oldest first.
    pub fn pauses(&self) -> &[Pause] {
        &self.pauses
    }

    /// Stop accruing study time.
    ///
    /// Fails if the timer is already paused or has finished; in either case the
    /// timer is left exactly as it was.
    pub fn pause(&mut self, clock: &dyn Clock) -> Result<(), TimerError> {
        match self.phase {
            Phase::Running => {
                self.phase = Phase::Paused { since: clock.now() };
                Ok(())
            }
            Phase::Paused { .. } => Err(TimerError::AlreadyPaused),
            Phase::Finished { .. } => Err(TimerError::AlreadyFinished),
        }
    }

    /// Start accruing study time again, closing the open pause.
    ///
    /// Fails if the timer is not paused; the timer is left exactly as it was.
    pub fn resume(&mut self, clock: &dyn Clock) -> Result<(), TimerError> {
        match self.phase {
            Phase::Paused { since } => {
                self.close_pause(since, clock.now());
                self.phase = Phase::Running;
                Ok(())
            }
            Phase::Running => Err(TimerError::NotPaused),
            Phase::Finished { .. } => Err(TimerError::AlreadyFinished),
        }
    }

    /// End the session, freezing its elapsed time.
    ///
    /// Finishing while paused closes the open pause at the finish time, so a
    /// session never carries a pause that stays open forever.
    pub fn finish(&mut self, clock: &dyn Clock) -> Result<(), TimerError> {
        let now = clock.now();
        match self.phase {
            Phase::Running => {
                self.phase = Phase::Finished { at: now };
                Ok(())
            }
            Phase::Paused { since } => {
                self.close_pause(since, now);
                self.phase = Phase::Finished { at: now };
                Ok(())
            }
            Phase::Finished { .. } => Err(TimerError::AlreadyFinished),
        }
    }

    /// Study time so far: wall-clock time since the start, minus every pause.
    ///
    /// Constant once the timer has finished, so the `clock` is then unused.
    pub fn elapsed(&self, clock: &dyn Clock) -> TimeDelta {
        let end = self.end(clock);
        non_negative(end - self.started_at - self.paused_until(end))
    }

    /// Total time spent paused, including a pause that is still open.
    pub fn paused(&self, clock: &dyn Clock) -> TimeDelta {
        self.paused_until(self.end(clock))
    }

    /// The instant durations are measured up to: the finish time if the timer
    /// has finished, otherwise now.
    fn end(&self, clock: &dyn Clock) -> DateTime<Utc> {
        match self.phase {
            Phase::Finished { at } => at,
            _ => clock.now(),
        }
    }

    /// Time paused up to `end`, counting an open pause as running until then.
    fn paused_until(&self, end: DateTime<Utc>) -> TimeDelta {
        let closed: TimeDelta = self.pauses.iter().map(Pause::duration).sum();
        let open = match self.phase {
            Phase::Paused { since } => non_negative(end - since),
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
