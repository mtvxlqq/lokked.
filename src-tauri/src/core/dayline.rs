//! The day timeline: which "study day" a moment belongs to, and how an
//! interval of time splits across study-day boundaries.
//!
//! A study day is not the calendar day: it starts at local midnight plus a
//! configurable offset (`day_start`), so a student who studies until 2 a.m.
//! can treat that time as still belonging to yesterday. Nothing here reads
//! the system clock or the system timezone — both come in as parameters, so
//! the module stays a pure function of its arguments and is trivially
//! testable.
//!
//! The timezone is `Tz: chrono::TimeZone` rather than a bespoke trait:
//! `chrono::FixedOffset` and `chrono::Utc` already implement it, and so does
//! `chrono::Local` (via the `iana-time-zone` feature chrono already pulls
//! in), which gives `platform/` a real, DST-aware system timezone for free
//! once it needs one — no adapter, no new dependency.
//!
//! Behavioural tests live in `src-tauri/tests/dayline.rs`.

use chrono::{DateTime, NaiveTime, Offset, TimeDelta, TimeZone, Utc};
use serde::{Deserialize, Serialize};

/// A slice of a `[start, end)` interval that falls entirely within one
/// study day, as defined by a timezone and a day-start offset.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Segment {
    /// 'YYYY-MM-DD', the study day this slice belongs to.
    pub day_key: String,
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
}

impl Segment {
    /// `end - start`. Never negative: `split_by_day` only ever produces
    /// segments where `end >= start`.
    pub fn duration(&self) -> TimeDelta {
        self.end - self.start
    }
}

/// Keeps `day_start` within `[0, 24h)` so a misconfigured negative or
/// over-a-day offset can't send the arithmetic below off the rails. Callers
/// never see an error for this — a day-start setting is validated at the UI
/// boundary, not here.
fn normalize_day_start(day_start: TimeDelta) -> TimeDelta {
    TimeDelta::seconds(day_start.num_seconds().rem_euclid(24 * 60 * 60))
}

/// Local wall-clock time at `moment_utc`, shifted back by `day_start` so
/// that taking just the date of the result gives the study day directly.
fn shifted_local<Tz: TimeZone>(
    moment_utc: DateTime<Utc>,
    tz: &Tz,
    day_start: TimeDelta,
) -> chrono::NaiveDateTime {
    let offset = tz.offset_from_utc_datetime(&moment_utc.naive_utc());
    moment_utc.naive_utc() + offset.fix() - normalize_day_start(day_start)
}

/// Which study day `moment_utc` falls into, as `'YYYY-MM-DD'` in `tz`, given
/// that a new study day begins at local midnight + `day_start`.
pub fn day_key<Tz: TimeZone>(moment_utc: DateTime<Utc>, tz: &Tz, day_start: TimeDelta) -> String {
    shifted_local(moment_utc, tz, day_start)
        .date()
        .format("%Y-%m-%d")
        .to_string()
}

/// The next instant, strictly after `moment_utc`, at which the study day
/// changes in `tz`. Always `> moment_utc`, even when `moment_utc` is itself
/// exactly a boundary — that instant belongs to the day it starts, not the
/// one before it.
pub fn next_boundary<Tz: TimeZone>(
    moment_utc: DateTime<Utc>,
    tz: &Tz,
    day_start: TimeDelta,
) -> DateTime<Utc> {
    let day_start = normalize_day_start(day_start);
    let day_start_time = NaiveTime::MIN + day_start;

    let target_date = shifted_local(moment_utc, tz, day_start).date() + TimeDelta::days(1);
    let target_local = target_date.and_time(day_start_time);

    // Inverting local -> UTC requires guessing an offset and refining it,
    // since `TimeZone::offset_from_local_datetime` can be ambiguous right
    // around a DST transition while `offset_from_utc_datetime` never is.
    // A single transition converges in at most two refinements; the extra
    // iterations are defensive slack, not a correctness requirement.
    let mut offset = tz.offset_from_utc_datetime(&moment_utc.naive_utc()).fix();
    for _ in 0..4 {
        let candidate = target_local - offset;
        let refined = tz.offset_from_utc_datetime(&candidate).fix();
        if refined == offset {
            return DateTime::<Utc>::from_naive_utc_and_offset(candidate, Utc);
        }
        offset = refined;
    }
    DateTime::<Utc>::from_naive_utc_and_offset(target_local - offset, Utc)
}

/// Splits `[start, end)` into one [`Segment`] per study day it crosses.
/// Returns an empty `Vec` if `start >= end`.
pub fn split_by_day<Tz: TimeZone>(
    start: DateTime<Utc>,
    end: DateTime<Utc>,
    tz: &Tz,
    day_start: TimeDelta,
) -> Vec<Segment> {
    let mut segments = Vec::new();
    let mut cursor = start;

    while cursor < end {
        let key = day_key(cursor, tz, day_start);
        let boundary = next_boundary(cursor, tz, day_start);
        let segment_end = boundary.min(end);

        segments.push(Segment {
            day_key: key,
            start: cursor,
            end: segment_end,
        });

        cursor = segment_end;
    }

    segments
}
