//! The statistics screen: time, cards, and one card's history.
//!
//! Three tabs, three commands, one period. Everything is derived from the
//! append-only `sessions` and `reviews` tables — nothing here writes, and
//! nothing here counts anything the pure reducers in
//! [`crate::core::stats`] could count instead.
//!
//! Which study day «сегодня» is comes from [`crate::core::dayline`], as
//! everywhere else: a student who works past midnight sees that time under
//! the day it belonged to, not under the calendar date SQLite would pick.

use chrono::Local;
use serde::Serialize;

use crate::commands::settings::day_start;
use crate::core::clock::Clock;
use crate::core::dayline::day_key;
use crate::core::stats::time::StatsRange;
use crate::db::reviews::ReviewRepo;
use crate::db::sessions::SessionRepo;
use crate::db::Database;
use crate::platform::clock::SystemClock;

use super::CommandError;

pub mod cards;
pub mod export;
pub mod time;

/// The days a tab is showing.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Period {
    pub range: StatsRange,
    /// First day of the period, `'YYYY-MM-DD'`.
    pub from: String,
    /// Last day — always today, so a period never runs into the future.
    pub to: String,
}

/// Works out which days `range` covers, ending at `today`.
///
/// «Всё время» has to ask the database where the history starts; the other
/// periods are arithmetic on the day key and touch nothing.
pub fn period(db: &Database, range: StatsRange, today: &str) -> Result<Period, CommandError> {
    let earliest = match range {
        StatsRange::All => earliest_day(db)?,
        _ => None,
    };

    Ok(Period {
        range,
        from: crate::core::stats::time::range_start(range, today, earliest.as_deref()),
        to: today.to_string(),
    })
}

/// The first day anything was recorded on, sessions and answers alike.
fn earliest_day(db: &Database) -> Result<Option<String>, CommandError> {
    let sessions = SessionRepo::new(db).earliest_day()?;
    let reviews = ReviewRepo::new(db).earliest_day()?;

    Ok(match (sessions, reviews) {
        (Some(left), Some(right)) => Some(left.min(right)),
        (only, None) | (None, only) => only,
    })
}

/// Which study day it is right now, by the student's own day boundary.
pub fn today(db: &Database) -> Result<String, CommandError> {
    Ok(day_key(SystemClock.now(), &Local, day_start(db)?))
}
