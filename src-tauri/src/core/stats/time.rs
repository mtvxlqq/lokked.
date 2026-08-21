//! The time side of the statistics screen: which period is being looked at,
//! how it splits between subjects, and the activity heatmap under it.
//!
//! Pure reducers over `(day_key, seconds)` and `(subject_id, seconds)` rows.
//! Which day is «today» is always a parameter — nothing here reads a clock,
//! and nothing here knows about timezones: by the time rows carry a
//! `day_key`, [`crate::core::dayline`] has already decided which study day
//! each moment belonged to.

use std::collections::HashMap;
use std::fmt;

use chrono::{Datelike, NaiveDate, TimeDelta};
use serde::{Deserialize, Serialize};

/// The darkest step of the heatmap. Levels run `0..=HEAT_LEVELS`, where 0 is
/// «не занимался» and is drawn as an empty cell rather than as a pale one.
pub const HEAT_LEVELS: u8 = 4;

/// How far back the статистика can look.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StatsRange {
    /// Сегодняшний учебный день.
    Day,
    /// Сегодня и шесть дней до него.
    Week,
    /// Тридцать дней, кончая сегодняшним.
    Month,
    /// С первого дня, по которому вообще что-то записано.
    All,
}

/// A period the statistics screen asked for that this module does not know.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnknownRange(pub String);

impl fmt::Display for UnknownRange {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "неизвестный период: {}", self.0)
    }
}

impl std::error::Error for UnknownRange {}

impl StatsRange {
    /// The slug the frontend sends.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Day => "day",
            Self::Week => "week",
            Self::Month => "month",
            Self::All => "all",
        }
    }

    pub fn parse(raw: &str) -> Result<Self, UnknownRange> {
        match raw {
            "day" => Ok(Self::Day),
            "week" => Ok(Self::Week),
            "month" => Ok(Self::Month),
            "all" => Ok(Self::All),
            other => Err(UnknownRange(other.to_string())),
        }
    }

    /// How many days the period covers, or `None` for «всё время».
    pub fn days(self) -> Option<i64> {
        match self {
            Self::Day => Some(1),
            Self::Week => Some(7),
            Self::Month => Some(30),
            Self::All => None,
        }
    }
}

/// The first day of `range`, given today and the earliest day on record.
///
/// The period always ends today, so «неделя» is today and the six days
/// before it, not the calendar week. `earliest` only matters for
/// [`StatsRange::All`]; a missing or unparsable one — and one somehow in the
/// future, which a wrong clock on another device could produce — collapses
/// the period to today rather than turning it inside out.
pub fn range_start(range: StatsRange, today: &str, earliest: Option<&str>) -> String {
    let Some(days) = range.days() else {
        return match (parse_day(today), earliest.and_then(parse_day)) {
            (Some(end), Some(first)) if first < end => first.format("%Y-%m-%d").to_string(),
            _ => today.to_string(),
        };
    };

    shift_day(today, -(days - 1))
}

/// A day key `days` days later — or earlier, for a negative `days`.
///
/// An unparsable key is handed back untouched: a period that cannot be
/// worked out degenerates to a single day rather than to an error, and the
/// screen shows one empty day instead of a failure.
pub fn shift_day(day_key: &str, days: i64) -> String {
    match parse_day(day_key) {
        Some(date) => (date + TimeDelta::days(days))
            .format("%Y-%m-%d")
            .to_string(),
        None => day_key.to_string(),
    }
}

/// The Monday `weeks - 1` weeks before the week `today` falls into.
///
/// The heatmap is drawn a column per week, so it has to start on a weekday
/// boundary — otherwise every column but the first is a week shifted by a
/// day or two, and the rows stop meaning anything.
pub fn heatmap_start(today: &str, weeks: i64) -> String {
    let Some(end) = parse_day(today) else {
        return today.to_string();
    };

    let monday = end - TimeDelta::days(i64::from(end.weekday().num_days_from_monday()));

    (monday - TimeDelta::weeks(weeks.max(1) - 1))
        .format("%Y-%m-%d")
        .to_string()
}

/// How long one subject was studied over the period.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SubjectTotal {
    pub subject_id: String,
    pub seconds: i64,
    /// Share of the longest bar, 0..=100 — the length to draw, not a share
    /// of the total: a bar chart is read by comparing bars with each other.
    pub share_percent: u32,
}

/// Time per subject over a period, longest first.
///
/// Rows for the same subject are summed, so the caller can pass a day's
/// worth and a period's worth alike. A subject with nothing recorded is
/// absent rather than present with a zero: the screen already has the
/// subject list, and a row of empty bars says nothing.
pub fn subject_totals(rows: &[(String, i64)]) -> Vec<SubjectTotal> {
    let mut by_subject: HashMap<&str, i64> = HashMap::new();
    for (subject_id, seconds) in rows {
        *by_subject.entry(subject_id.as_str()).or_insert(0) += seconds;
    }

    let mut totals: Vec<(&str, i64)> = by_subject
        .into_iter()
        .filter(|(_, seconds)| *seconds > 0)
        .collect();

    // По убыванию времени, при равенстве — по id: одинаковые данные всегда
    // дают одинаковый порядок, иначе строки прыгали бы между перерисовками.
    totals.sort_by(|left, right| right.1.cmp(&left.1).then(left.0.cmp(right.0)));

    let longest = totals.first().map(|(_, seconds)| *seconds).unwrap_or(0);

    totals
        .into_iter()
        .map(|(subject_id, seconds)| SubjectTotal {
            subject_id: subject_id.to_string(),
            seconds,
            share_percent: if longest > 0 {
                (seconds * 100 / longest) as u32
            } else {
                0
            },
        })
        .collect()
}

/// One day of the activity heatmap.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct HeatCell {
    pub day_key: String,
    pub seconds: i64,
    /// 0 for a day without study, otherwise 1..=[`HEAT_LEVELS`], measured
    /// against the best day of the same period.
    pub level: u8,
    /// Monday is 0. The grid is drawn a column per week, and this is which
    /// row the cell lands in.
    pub weekday: u8,
}

/// A cell for every day of `[from, to]`, days without study included.
///
/// Levels are relative to the best day of the period rather than to a fixed
/// number of hours: what counts as a good day differs between a school
/// holiday and an exam week, and the picture is read by comparing days with
/// each other. Any day with study at all gets at least level 1 — the
/// difference between «десять минут» and «ничего» is the whole point.
pub fn heatmap(days: &[(String, i64)], from: &str, to: &str) -> Vec<HeatCell> {
    let span = day_span(from, to);
    if span.is_empty() {
        return Vec::new();
    }

    let mut by_day: HashMap<&str, i64> = HashMap::new();
    for (day_key, seconds) in days {
        *by_day.entry(day_key.as_str()).or_insert(0) += seconds;
    }

    let cells: Vec<(NaiveDate, String, i64)> = span
        .into_iter()
        .map(|date| {
            let key = date.format("%Y-%m-%d").to_string();
            let seconds = by_day.get(key.as_str()).copied().unwrap_or(0).max(0);
            (date, key, seconds)
        })
        .collect();

    let best = cells
        .iter()
        .map(|(_, _, seconds)| *seconds)
        .max()
        .unwrap_or(0);

    cells
        .into_iter()
        .map(|(date, day_key, seconds)| HeatCell {
            day_key,
            seconds,
            level: heat_level(seconds, best),
            weekday: date.weekday().num_days_from_monday() as u8,
        })
        .collect()
}

/// Which step of the scale `seconds` lands on, rounding up so that nothing
/// studied ever reads as nothing at all.
fn heat_level(seconds: i64, best: i64) -> u8 {
    if seconds <= 0 || best <= 0 {
        return 0;
    }

    let levels = i64::from(HEAT_LEVELS);
    let step = (seconds * levels + best - 1) / best;

    step.clamp(1, levels) as u8
}

/// Every day of `[from, to]`, in order. Empty if either end is unparsable or
/// the period runs backwards.
pub fn day_span(from: &str, to: &str) -> Vec<NaiveDate> {
    let (Some(first), Some(last)) = (parse_day(from), parse_day(to)) else {
        return Vec::new();
    };
    if last < first {
        return Vec::new();
    }

    let mut days = Vec::new();
    let mut cursor = first;
    while cursor <= last {
        days.push(cursor);
        cursor += TimeDelta::days(1);
    }

    days
}

/// A `'YYYY-MM-DD'` day key as a date.
pub fn parse_day(day_key: &str) -> Option<NaiveDate> {
    NaiveDate::parse_from_str(day_key, "%Y-%m-%d").ok()
}
