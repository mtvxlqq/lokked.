//! The study streak: how many days in a row the student has put in enough
//! time.
//!
//! A pure reducer over `(day_key, seconds)` pairs — nothing here reads the
//! clock, so which day is «today» is a parameter and a test can pick any.

use std::collections::HashMap;

use chrono::{NaiveDate, TimeDelta};

/// How much has to be studied for a day to count toward the streak.
///
/// Ten minutes: enough that opening the app and closing it does not count,
/// low enough that a genuinely busy day can still be kept alive.
pub const STREAK_MIN_SECONDS: i64 = 10 * 60;

/// How far back a streak is counted.
///
/// Long enough that no realistic streak is cut short, short enough that the
/// query behind it stays a scan of one small index rather than of the whole
/// history.
pub const STREAK_WINDOW_DAYS: i64 = 400;

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
