//! The «Время» tab: how much was studied, by which subject, and on which
//! days.
//!
//! Only what has been written down is counted. A phase running right now is
//! not part of any of these figures — it has no row yet, and the main screen
//! is where the time in progress is shown. The moment it ends it is
//! recorded, and the numbers here move.

use serde::Serialize;
use tauri::State;

use crate::core::stats::time::{
    heatmap, heatmap_start, shift_day, subject_totals, HeatCell, StatsRange, SubjectTotal,
};
use crate::core::stats::{streak, STREAK_WINDOW_DAYS};
use crate::db::sessions::SessionRepo;
use crate::db::Database;

use super::{period, today, CommandError, Period};

/// How many weeks the activity heatmap shows.
///
/// Thirty is what fits a desktop window without shrinking the cells below a
/// readable size, and it is deliberately independent of the selected period:
/// the picture is there to show the shape of a habit over months, which a
/// «за сегодня» heatmap of one cell could not.
pub const HEATMAP_WEEKS: i64 = 30;

/// Everything the «Время» tab draws.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct TimeStats {
    #[serde(flatten)]
    pub period: Period,
    /// Everything studied over the period, in seconds — breaks excluded.
    pub total_seconds: i64,
    /// Pomodoro work phases carried to their end over the period.
    pub pomodoros: i64,
    /// Days in a row, counted from today — the same figure the main screen
    /// shows, and just as independent of the selected period.
    pub streak_days: u32,
    /// Time per subject, longest first.
    pub subjects: Vec<SubjectTotal>,
    /// The last [`HEATMAP_WEEKS`] weeks, a cell per day, starting on a
    /// Monday.
    pub heatmap: Vec<HeatCell>,
}

/// The «Время» tab for `range`, as of the study day `today`.
pub fn time_stats(
    db: &Database,
    range: StatsRange,
    today: &str,
) -> Result<TimeStats, CommandError> {
    let period = period(db, range, today)?;
    let repo = SessionRepo::new(db);

    let subjects = subject_totals(&repo.active_seconds_by_subject_range(&period.from, &period.to)?);

    let heat_from = heatmap_start(today, HEATMAP_WEEKS);
    let heat_days = repo.active_seconds_by_day(&heat_from, today)?;

    // Серия считается по своему окну: она кончается сегодня и может быть
    // длиннее любого выбранного периода — «за неделю» не значит «серия не
    // больше семи».
    let streak_window = shift_day(today, -STREAK_WINDOW_DAYS);
    let streak_days = streak(&repo.active_seconds_by_day(&streak_window, today)?, today);

    Ok(TimeStats {
        total_seconds: subjects.iter().map(|subject| subject.seconds).sum(),
        pomodoros: repo.completed_pomodoros_range(&period.from, &period.to)?,
        streak_days,
        subjects,
        heatmap: heatmap(&heat_days, &heat_from, today),
        period,
    })
}

#[tauri::command]
pub fn stats_time(db: State<'_, Database>, range: String) -> Result<TimeStats, CommandError> {
    let range = StatsRange::parse(&range)?;
    let today = today(&db)?;

    time_stats(&db, range, &today)
}
