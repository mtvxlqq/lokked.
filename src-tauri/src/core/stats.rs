//! Aggregations over study sessions and card reviews.
//!
//! Pure reducers: rows in, numbers out. Nothing here reads the clock or the
//! database — the caller passes both the records and which day «today» is.
//!
//! TODO: the per-subject breakdowns the statistics screen needs (M12).

use std::collections::HashMap;

use chrono::{NaiveDate, TimeDelta};
use serde::Serialize;

use super::review::Grade;

/// How much has to be studied for a day to count toward the streak.
///
/// Ten minutes: enough that opening the app and closing it does not count,
/// low enough that a genuinely busy day can still be kept alive.
pub const STREAK_MIN_SECONDS: i64 = 10 * 60;

/// How many days in a row, ending today, the student studied at least
/// [`STREAK_MIN_SECONDS`].
///
/// `days` is `(day_key, active_seconds)` in any order and with repeats — the
/// seconds of one day are summed before the threshold is applied.
///
/// A day that has only just begun does not break anything: if today is not
/// counted yet, the streak is measured up to yesterday instead. That is what
/// makes the number survive midnight, which is the whole point of a streak.
pub fn streak(days: &[(String, i64)], today: &str) -> u32 {
    let Ok(today) = NaiveDate::parse_from_str(today, "%Y-%m-%d") else {
        // Без сегодняшнего дня не от чего отсчитывать.
        return 0;
    };

    let mut by_day: HashMap<NaiveDate, i64> = HashMap::new();
    for (key, seconds) in days {
        let Ok(date) = NaiveDate::parse_from_str(key, "%Y-%m-%d") else {
            continue;
        };
        if date > today {
            continue;
        }
        *by_day.entry(date).or_insert(0) += seconds;
    }

    let counted = |date: NaiveDate| by_day.get(&date).is_some_and(|s| *s >= STREAK_MIN_SECONDS);

    let mut cursor = if counted(today) {
        today
    } else {
        today - TimeDelta::days(1)
    };

    let mut length = 0;
    while counted(cursor) {
        length += 1;
        cursor -= TimeDelta::days(1);
    }

    length
}

/// One answered card, as a finished run remembers it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReviewOutcome {
    pub card_id: String,
    pub grade: Grade,
    /// Time from the card appearing to the grade being given.
    pub total_ms: i64,
}

/// What the screen after a run shows.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct ReviewSummary {
    pub answered: usize,
    pub correct: usize,
    /// Rounded to the nearest per cent; zero for a run with no answers.
    pub accuracy_percent: u32,
    pub total_ms: i64,
    /// Mean time per card, rounded; zero for a run with no answers.
    pub average_ms: i64,
    /// Cards answered «не помню», in the order they came up. Not deduplicated:
    /// a card answered twice in one run really was missed twice.
    pub mistakes: Vec<String>,
}

/// Turns the answers of one run into the numbers under it.
pub fn review_summary(results: &[ReviewOutcome]) -> ReviewSummary {
    let answered = results.len();
    if answered == 0 {
        return ReviewSummary::default();
    }

    let correct = results
        .iter()
        .filter(|result| result.grade.is_correct())
        .count();
    let total_ms: i64 = results.iter().map(|result| result.total_ms).sum();

    ReviewSummary {
        answered,
        correct,
        // Целочисленное округление к ближайшему: 2 из 3 — это 67%, а не 66%.
        accuracy_percent: ((correct * 200 + answered) / (answered * 2)) as u32,
        total_ms,
        average_ms: (total_ms + answered as i64 / 2) / answered as i64,
        mistakes: results
            .iter()
            .filter(|result| !result.grade.is_correct())
            .map(|result| result.card_id.clone())
            .collect(),
    }
}
