//! The «сегодня» figures the subject list shows next to each subject.
//!
//! Which day «today» is depends on the timezone and on where the student put
//! the day boundary, so the answer is computed with
//! [`crate::core::dayline::day_key`] rather than by trusting SQLite's `date()`
//! — the two disagree for anyone studying past midnight.

use chrono::{Local, TimeDelta};
use serde::Serialize;
use tauri::State;

use crate::core::clock::Clock;
use crate::core::dayline::day_key;
use crate::db::sessions::SessionRepo;
use crate::db::Database;
use crate::platform::clock::SystemClock;

use super::CommandError;

/// How much was studied today, per subject.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct TodayTotals {
    /// The study day these totals are for, `'YYYY-MM-DD'`. The frontend keeps
    /// it so it can tell a stale summary from a fresh one after the day rolls
    /// over while the window was open.
    pub day_key: String,
    /// `(subject_id, seconds)`, only for subjects with time recorded today.
    pub seconds_by_subject: Vec<(String, i64)>,
}

/// Study time recorded today, per subject.
///
/// `day_start` is an offset from local midnight; the setting that will carry
/// it is part of M8, so today it is always zero.
pub fn totals(
    db: &Database,
    clock: &dyn Clock,
    day_start: TimeDelta,
) -> Result<TodayTotals, CommandError> {
    let key = day_key(clock.now(), &Local, day_start);
    let seconds_by_subject = SessionRepo::new(db).active_seconds_by_subject(&key)?;

    Ok(TodayTotals {
        day_key: key,
        seconds_by_subject,
    })
}

#[tauri::command]
pub fn today_totals(db: State<'_, Database>) -> Result<TodayTotals, CommandError> {
    totals(&db, &SystemClock, TimeDelta::zero())
}
