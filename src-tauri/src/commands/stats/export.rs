//! «Экспорт в CSV»: the period as a table, a line per day.
//!
//! The text is handed back rather than written to a file, exactly as the
//! deck export is: a save dialog is a platform permission of its own, and
//! what it would buy over «скопировать» is a file the student then has to
//! find.

use tauri::State;

use crate::core::stats::csv::{daily_report, DailyRow};
use crate::core::stats::time::{day_span, StatsRange};
use crate::db::reviews::ReviewRepo;
use crate::db::sessions::SessionRepo;
use crate::db::Database;

use super::{period, today, CommandError};

/// The period as CSV: day, seconds, minutes, cards answered, of them
/// correct, accuracy.
pub fn export_csv(db: &Database, range: StatsRange, today: &str) -> Result<String, CommandError> {
    let period = period(db, range, today)?;

    let seconds = SessionRepo::new(db).active_seconds_by_day(&period.from, &period.to)?;
    let counts = ReviewRepo::new(db).counts_by_day(&period.from, &period.to)?;

    let rows: Vec<DailyRow> = day_span(&period.from, &period.to)
        .into_iter()
        .map(|date| {
            let day_key = date.format("%Y-%m-%d").to_string();
            let studied: i64 = seconds
                .iter()
                .filter(|(day, _)| *day == day_key)
                .map(|(_, total)| *total)
                .sum();
            let (answered, correct) = counts
                .iter()
                .filter(|(day, _, _)| *day == day_key)
                .fold((0, 0), |(answered, correct), (_, a, c)| {
                    (answered + a, correct + c)
                });

            DailyRow {
                day_key,
                seconds: studied,
                answered,
                correct,
            }
        })
        .collect();

    Ok(daily_report(&rows))
}

#[tauri::command]
pub fn stats_export_csv(db: State<'_, Database>, range: String) -> Result<String, CommandError> {
    let range = StatsRange::parse(&range)?;
    let today = today(&db)?;

    export_csv(&db, range, &today)
}
