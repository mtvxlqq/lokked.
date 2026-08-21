//! Turning a finished timer phase into rows for `sessions`, and judging how
//! long the app was away.
//!
//! The timer knows *when* things happened; this module answers the two
//! questions that come up when a phase ends. Which study days did it touch,
//! and how much of each day was actually studied rather than paused? A phase
//! that runs past the day boundary is stored as one row per day — CLAUDE.md
//! rule 4: nothing is deleted at midnight, «сегодня» is a filter on
//! `day_key`, so the split has to happen when the row is written.
//!
//! Behavioural tests live in `src-tauri/tests/session_slicing.rs`.

use chrono::{DateTime, TimeDelta, TimeZone, Utc};
use serde::{Deserialize, Serialize};

use super::dayline::split_by_day;
use super::timer::Pause;

/// One row of `sessions`: the part of a phase that fell within one study day.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionSlice {
    /// `'YYYY-MM-DD'` — the study day this part belongs to.
    pub day_key: String,
    pub started_at: DateTime<Utc>,
    pub ended_at: DateTime<Utc>,
    /// Wall-clock time in this slice, minus the pauses inside it.
    pub active_seconds: i64,
    /// Time paused within this slice.
    pub paused_seconds: i64,
}

/// How long `pause` and `[from, to)` overlap. Zero when they do not.
fn overlap(pause: &Pause, from: DateTime<Utc>, to: DateTime<Utc>) -> TimeDelta {
    let start = pause.started_at.max(from);
    let end = pause.ended_at.min(to);
    (end - start).max(TimeDelta::zero())
}

/// Splits one finished phase into the rows that describe it.
///
/// Returns an empty `Vec` for an empty phase — including one whose end is
/// before its start, which a backwards clock jump can produce. A slice with
/// zero active seconds is *not* dropped: the session really did span that
/// day, and dropping it would lose the pause from the statistics.
pub fn slice_phase<Tz: TimeZone>(
    started_at: DateTime<Utc>,
    ended_at: DateTime<Utc>,
    pauses: &[Pause],
    tz: &Tz,
    day_start: TimeDelta,
) -> Vec<SessionSlice> {
    split_by_day(started_at, ended_at, tz, day_start)
        .into_iter()
        .map(|segment| {
            let paused: TimeDelta = pauses
                .iter()
                .map(|pause| overlap(pause, segment.start, segment.end))
                .sum();
            let active = (segment.duration() - paused).max(TimeDelta::zero());

            SessionSlice {
                day_key: segment.day_key,
                started_at: segment.start,
                ended_at: segment.end,
                active_seconds: active.num_seconds(),
                paused_seconds: paused.num_seconds(),
            }
        })
        .collect()
}

/// How long the app may be out of sight before the time counts as suspect.
///
/// Below this, the student almost certainly just switched windows and came
/// back, and interrupting them to ask would be noise. Above it, the app was
/// probably in the background or the machine asleep, and only the student
/// knows whether that hour was studying.
pub const AWAY_PROMPT_SECONDS: i64 = 10 * 60;

/// What to do about time that passed while the app was not visible.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct AwayReport {
    /// Wall-clock seconds the app was away, never negative.
    pub away_seconds: i64,
    /// Whether the student should be asked to keep or discard that time.
    pub needs_decision: bool,
}

/// Judges an absence between `last_seen` and `now`.
///
/// A clock that moved backwards reports zero rather than a negative absence:
/// an NTP correction is not time the student spent away.
pub fn away_report(last_seen: DateTime<Utc>, now: DateTime<Utc>) -> AwayReport {
    let away_seconds = (now - last_seen).max(TimeDelta::zero()).num_seconds();

    AwayReport {
        away_seconds,
        needs_decision: away_seconds >= AWAY_PROMPT_SECONDS,
    }
}
