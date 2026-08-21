//! The «сегодня» summary the subject list shows: how much was studied today,
//! by whom, how many Pomodoros were finished, and how long the streak is.
//!
//! Which day «today» is depends on the timezone and on where the student put
//! the day boundary, so the answer is computed with
//! [`crate::core::dayline::day_key`] rather than by trusting SQLite's `date()`
//! — the two disagree for anyone studying past midnight.
//!
//! Nothing is ever deleted at the boundary (CLAUDE.md rule 4): a new day
//! simply means a different `day_key` to filter by, and yesterday's rows stay
//! exactly where they are.

use chrono::{DateTime, Local, TimeDelta, Utc};
use serde::Serialize;
use tauri::State;

use crate::commands::session::{work_in_progress, SessionState};
use crate::commands::settings::day_start;
use crate::core::clock::Clock;
use crate::core::dayline::{day_key, next_boundary};
use crate::core::stats::{streak, STREAK_WINDOW_DAYS};
use crate::db::sessions::SessionRepo;
use crate::db::Database;
use crate::platform::clock::SystemClock;

use super::CommandError;

/// What the subject list shows above the subjects.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct TodayTotals {
    /// The study day these totals are for, `'YYYY-MM-DD'`. The frontend keeps
    /// it so it can tell a stale summary from a fresh one after the day rolls
    /// over while the window was open.
    pub day_key: String,
    /// `(subject_id, seconds)`, only for subjects with time recorded today.
    pub seconds_by_subject: Vec<(String, i64)>,
    /// Everything studied today, in seconds — breaks excluded.
    pub total_seconds: i64,
    /// Pomodoro work phases carried to their end today.
    pub pomodoros: i64,
    /// Days in a row with at least [`crate::core::stats::STREAK_MIN_SECONDS`].
    pub streak_days: u32,
    /// When the study day changes next. The frontend arms a timer for it and
    /// reloads, so a window left open overnight does not keep showing
    /// yesterday.
    pub next_boundary: DateTime<Utc>,
}

/// The summary for the study day `now` falls into.
///
/// The moment is a parameter rather than a clock because the caller has
/// already had to ask what day it is — to work out the running phase's share
/// of it — and asking twice could, at the boundary itself, file the two
/// halves of the answer against different days.
///
/// `in_progress` is `(subject_id, seconds)` from the phase that is running
/// and has not been written down yet; it is folded into every figure here,
/// because from the student's side that time has been studied whether or not
/// a row exists for it.
pub fn totals(
    db: &Database,
    now: DateTime<Utc>,
    day_start: TimeDelta,
    in_progress: Option<(String, i64)>,
) -> Result<TodayTotals, CommandError> {
    let key = day_key(now, &Local, day_start);
    let repo = SessionRepo::new(db);

    let mut seconds_by_subject = repo.active_seconds_by_subject(&key)?;
    if let Some((subject_id, seconds)) = &in_progress {
        match seconds_by_subject
            .iter_mut()
            .find(|(id, _)| id == subject_id)
        {
            Some((_, total)) => *total += seconds,
            None => seconds_by_subject.push((subject_id.clone(), *seconds)),
        }
    }

    let window_start = day_key(now - TimeDelta::days(STREAK_WINDOW_DAYS), &Local, day_start);
    let mut by_day = repo.active_seconds_by_day(&window_start, &key)?;
    if let Some((_, seconds)) = &in_progress {
        by_day.push((key.clone(), *seconds));
    }

    Ok(TodayTotals {
        total_seconds: seconds_by_subject.iter().map(|(_, s)| s).sum(),
        pomodoros: repo.completed_pomodoros(&key)?,
        streak_days: streak(&by_day, &key),
        next_boundary: next_boundary(now, &Local, day_start),
        day_key: key,
        seconds_by_subject,
    })
}

#[tauri::command]
pub fn today_totals(
    db: State<'_, Database>,
    session: State<'_, SessionState>,
) -> Result<TodayTotals, CommandError> {
    let now = SystemClock.now();
    let day_start = day_start(&db)?;
    let key = day_key(now, &Local, day_start);
    let running = work_in_progress(&session, &SystemClock, day_start, &key);

    totals(&db, now, day_start, running)
}
