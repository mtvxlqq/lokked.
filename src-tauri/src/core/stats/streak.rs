//! The study streak: how many days in a row the student has put in enough
//! time, what keeps it alive across a missed day, and how the days are
//! marked for a calendar.
//!
//! A pure reducer over `(day_key, seconds)` pairs — nothing here reads the
//! clock, so which day is «today» is a parameter and a test can pick any.
//!
//! The one rule that shapes everything else: **the streak is not reset at
//! midnight**. A day that has only just begun is not a miss, it is a day
//! that has not happened yet, so the number survives until the day is over.

use std::collections::HashMap;

use chrono::{Datelike, NaiveDate, TimeDelta};
use serde::Serialize;

/// How much has to be studied for a day to count toward the streak.
///
/// Ten minutes: enough that opening the app and closing it does not count,
/// low enough that a genuinely busy day can still be kept alive. It is the
/// default of a setting, not a constant of the domain — see
/// [`crate::core::settings::StreakSettings`].
pub const STREAK_MIN_SECONDS: i64 = 10 * 60;

/// How far back a streak is counted.
///
/// Long enough that no realistic streak is cut short, short enough that the
/// query behind it stays a scan of one small index rather than of the whole
/// history.
pub const STREAK_WINDOW_DAYS: i64 = 400;

/// How many days in a row earn one freeze.
pub const FREEZE_EVERY_DAYS: u32 = 10;

/// How many freezes can be held at once.
///
/// Three is a safety net for a week of exams, not a way to keep a streak
/// without studying.
pub const MAX_FREEZES: u32 = 3;

/// The streaks worth aiming at, in days.
pub const MILESTONES: [u32; 3] = [7, 30, 100];

/// The rules a streak is counted by.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StreakRules {
    /// Seconds of study that make a day count.
    pub min_seconds: i64,
    /// Days in a row that earn one freeze.
    pub freeze_every: u32,
    /// How many freezes can be held at once.
    pub max_freezes: u32,
}

impl Default for StreakRules {
    fn default() -> Self {
        Self {
            min_seconds: STREAK_MIN_SECONDS,
            freeze_every: FREEZE_EVERY_DAYS,
            max_freezes: MAX_FREEZES,
        }
    }
}

/// What became of one day.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DayState {
    /// Studied enough. Counts towards the streak.
    Counted,
    /// Missed, but a freeze was spent on it, so the streak went on.
    Frozen,
    /// Missed with nothing left to spend. Ends the streak.
    Missed,
    /// Today, still short of the minimum — not a miss until it is over.
    Pending,
    /// A day of the calendar that has not arrived yet.
    Future,
}

/// One day of the run, as the calendar draws it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DayMark {
    pub day: String,
    pub seconds: i64,
    pub state: DayState,
}

/// Everything the streak page is built out of.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct StreakState {
    /// Days studied in a row right now, frozen days not counted.
    pub current: u32,
    /// The longest such run there has ever been.
    pub longest: u32,
    /// The days that run began and ended on.
    pub longest_from: Option<String>,
    pub longest_to: Option<String>,
    /// Freezes in hand.
    pub freezes: u32,
    /// Freezes spent inside the current streak.
    pub frozen_days: u32,
    /// Every day from the first one studied up to today.
    pub days: Vec<DayMark>,
}

/// A streak worth aiming at, and how far off it is.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Milestone {
    pub target: u32,
    pub reached: bool,
    /// The day the current streak passed it, if it has.
    pub reached_on: Option<String>,
    /// Days still to go, or zero once it is taken.
    pub remaining: u32,
}

/// How many days in a row, ending today, the student studied enough.
///
/// The short answer, for the screens that show the number next to something
/// else. [`streak_state`] is the same walk with everything it learned along
/// the way.
pub fn streak(days: &[(String, i64)], today: &str) -> u32 {
    streak_state(days, today, StreakRules::default()).current
}

/// Walks the days from the first one studied up to `today`.
///
/// `days` is `(day_key, active_seconds)` in any order and with repeats — the
/// seconds of one day are summed before the minimum is applied.
pub fn streak_state(days: &[(String, i64)], today: &str, rules: StreakRules) -> StreakState {
    let mut state = StreakState {
        current: 0,
        longest: 0,
        longest_from: None,
        longest_to: None,
        freezes: 0,
        frozen_days: 0,
        days: Vec::new(),
    };

    let Ok(today) = NaiveDate::parse_from_str(today, "%Y-%m-%d") else {
        // Без сегодняшнего дня не от чего отсчитывать.
        return state;
    };

    let by_day = totals(days, today);
    let Some(first) = by_day.keys().min().copied() else {
        return state;
    };

    let mut started_on: Option<NaiveDate> = None;
    let mut cursor = first;

    while cursor <= today {
        let seconds = by_day.get(&cursor).copied().unwrap_or(0);
        let day_state = if seconds >= rules.min_seconds {
            state.current += 1;
            started_on.get_or_insert(cursor);
            if state.current % rules.freeze_every == 0 {
                state.freezes = (state.freezes + 1).min(rules.max_freezes);
            }
            if state.current > state.longest {
                state.longest = state.current;
                state.longest_from = started_on.map(format_day);
                state.longest_to = Some(format_day(cursor));
            }
            DayState::Counted
        } else if cursor == today {
            // Сегодня ещё идёт: недобранные минуты — это не пропуск.
            DayState::Pending
        } else if state.current > 0 && state.freezes > 0 {
            state.freezes -= 1;
            state.frozen_days += 1;
            DayState::Frozen
        } else {
            // Серия кончилась, и запас кончился вместе с ней.
            state.current = 0;
            state.freezes = 0;
            state.frozen_days = 0;
            started_on = None;
            DayState::Missed
        };

        state.days.push(DayMark {
            day: format_day(cursor),
            seconds,
            state: day_state,
        });
        cursor += TimeDelta::days(1);
    }

    state
}

/// The milestones, measured against the streak as it stands.
///
/// A milestone already taken carries the day the current streak passed it —
/// which is a count of studied days, not of calendar ones: a frozen day
/// keeps the streak but does not move it forward.
pub fn milestones(state: &StreakState) -> Vec<Milestone> {
    let counted = current_run(state);

    MILESTONES
        .iter()
        .map(|target| {
            let reached = state.current >= *target;

            Milestone {
                target: *target,
                reached,
                reached_on: reached
                    .then(|| counted.get(*target as usize - 1).cloned())
                    .flatten(),
                remaining: target.saturating_sub(state.current),
            }
        })
        .collect()
}

/// One calendar month, a mark per day, for the streak page's calendar.
///
/// Days the walk never reached are two different things and are marked as
/// such: a day before the student ever studied — or after a break — is a
/// plain miss, while a day still to come is [`DayState::Future`] and is
/// drawn as an empty cell rather than as a failure.
pub fn month_days(state: &StreakState, today: &str, year: i32, month: u32) -> Vec<DayMark> {
    let Some(first) = NaiveDate::from_ymd_opt(year, month, 1) else {
        return Vec::new();
    };
    let today = NaiveDate::parse_from_str(today, "%Y-%m-%d").unwrap_or(first);

    let mut days = Vec::new();
    let mut cursor = first;

    while cursor.month() == month {
        let day = format_day(cursor);
        days.push(match state.days.iter().find(|marked| marked.day == day) {
            Some(marked) => marked.clone(),
            None => DayMark {
                day,
                seconds: 0,
                state: if cursor > today {
                    DayState::Future
                } else {
                    DayState::Missed
                },
            },
        });
        cursor += TimeDelta::days(1);
    }

    days
}

/// The days studied inside the current streak, oldest first.
fn current_run(state: &StreakState) -> Vec<String> {
    let mut counted: Vec<String> = state
        .days
        .iter()
        .rev()
        .take_while(|day| day.state != DayState::Missed)
        .filter(|day| day.state == DayState::Counted)
        .map(|day| day.day.clone())
        .collect();
    counted.reverse();

    counted
}

/// Seconds per day, summed, with anything past `today` left out.
fn totals(days: &[(String, i64)], today: NaiveDate) -> HashMap<NaiveDate, i64> {
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

    by_day
}

fn format_day(date: NaiveDate) -> String {
    date.format("%Y-%m-%d").to_string()
}
