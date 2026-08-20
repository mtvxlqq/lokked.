//! Tests for `core::dayline`: which study day a moment belongs to, the next
//! day boundary, and splitting an interval across boundaries — including
//! daylight-saving transitions.

use chrono::{
    DateTime, FixedOffset, MappedLocalTime, NaiveDate, NaiveDateTime, TimeDelta, TimeZone, Utc,
};
use lokked_lib::core::dayline::{day_key, next_boundary, split_by_day, Segment};

fn at(y: i32, m: u32, d: u32, h: u32, mi: u32, s: u32) -> DateTime<Utc> {
    Utc.with_ymd_and_hms(y, m, d, h, mi, s).unwrap()
}

/// A timezone that switches offset once, at a fixed UTC instant — used to
/// simulate a DST transition deterministically, without touching the
/// system's real timezone database.
#[derive(Clone, Copy)]
struct TransitionZone {
    switch_at: DateTime<Utc>,
    before: FixedOffset,
    after: FixedOffset,
}

impl TimeZone for TransitionZone {
    type Offset = FixedOffset;

    fn from_offset(offset: &FixedOffset) -> Self {
        TransitionZone {
            switch_at: DateTime::<Utc>::UNIX_EPOCH,
            before: *offset,
            after: *offset,
        }
    }

    // Never called by `core::dayline`, which only ever inverts local time
    // through `offset_from_utc_datetime`. Present only to satisfy the trait.
    fn offset_from_local_date(&self, _local: &NaiveDate) -> MappedLocalTime<FixedOffset> {
        MappedLocalTime::Single(self.after)
    }

    fn offset_from_local_datetime(&self, _local: &NaiveDateTime) -> MappedLocalTime<FixedOffset> {
        MappedLocalTime::Single(self.after)
    }

    fn offset_from_utc_date(&self, utc: &NaiveDate) -> FixedOffset {
        self.offset_from_utc_datetime(&utc.and_hms_opt(0, 0, 0).unwrap())
    }

    fn offset_from_utc_datetime(&self, utc: &NaiveDateTime) -> FixedOffset {
        if *utc < self.switch_at.naive_utc() {
            self.before
        } else {
            self.after
        }
    }
}

fn segment(day: &str, start: DateTime<Utc>, end: DateTime<Utc>) -> Segment {
    Segment {
        day_key: day.to_string(),
        start,
        end,
    }
}

// --- day_key -----------------------------------------------------------

#[test]
fn moment_before_the_04_00_boundary_belongs_to_the_previous_day() {
    let key = day_key(at(2026, 8, 6, 3, 59, 59), &Utc, TimeDelta::hours(4));
    assert_eq!(key, "2026-08-05");
}

#[test]
fn moment_exactly_on_the_04_00_boundary_belongs_to_the_new_day() {
    let key = day_key(at(2026, 8, 6, 4, 0, 0), &Utc, TimeDelta::hours(4));
    assert_eq!(key, "2026-08-06");
}

#[test]
fn moment_just_after_the_04_00_boundary_belongs_to_the_new_day() {
    let key = day_key(at(2026, 8, 6, 4, 0, 1), &Utc, TimeDelta::hours(4));
    assert_eq!(key, "2026-08-06");
}

#[test]
fn day_key_respects_a_nonzero_fixed_offset() {
    let tz = FixedOffset::east_opt(3 * 3600).unwrap();

    // 2026-08-06T00:59:00Z is 03:59 local -> still the previous study day.
    assert_eq!(
        day_key(at(2026, 8, 6, 0, 59, 0), &tz, TimeDelta::hours(4)),
        "2026-08-05"
    );
    // 2026-08-06T01:00:00Z is 04:00 local -> the new study day.
    assert_eq!(
        day_key(at(2026, 8, 6, 1, 0, 0), &tz, TimeDelta::hours(4)),
        "2026-08-06"
    );
}

#[test]
fn zero_day_start_matches_the_plain_calendar_date() {
    let key = day_key(at(2026, 8, 6, 23, 59, 59), &Utc, TimeDelta::zero());
    assert_eq!(key, "2026-08-06");
}

// --- next_boundary -------------------------------------------------------

#[test]
fn next_boundary_moves_to_the_04_00_offset_boundary() {
    let boundary = next_boundary(at(2026, 8, 6, 1, 23, 0), &Utc, TimeDelta::hours(4));
    assert_eq!(boundary, at(2026, 8, 6, 4, 0, 0));
}

#[test]
fn next_boundary_from_exactly_on_a_boundary_is_the_following_one() {
    let boundary = next_boundary(at(2026, 8, 6, 4, 0, 0), &Utc, TimeDelta::hours(4));
    assert_eq!(boundary, at(2026, 8, 7, 4, 0, 0));
}

#[test]
fn next_boundary_just_before_midnight_lands_on_the_offset_boundary() {
    let boundary = next_boundary(at(2026, 8, 6, 23, 0, 0), &Utc, TimeDelta::hours(4));
    assert_eq!(boundary, at(2026, 8, 7, 4, 0, 0));
}

#[test]
fn next_boundary_with_default_midnight_day_start() {
    let boundary = next_boundary(at(2026, 8, 6, 12, 0, 0), &Utc, TimeDelta::zero());
    assert_eq!(boundary, at(2026, 8, 7, 0, 0, 0));
}

// --- split_by_day --------------------------------------------------------

#[test]
fn interval_within_a_single_day_is_one_segment() {
    let start = at(2026, 8, 6, 9, 0, 0);
    let end = at(2026, 8, 6, 10, 0, 0);

    let segments = split_by_day(start, end, &Utc, TimeDelta::zero());

    assert_eq!(segments, vec![segment("2026-08-06", start, end)]);
}

#[test]
fn interval_crossing_midnight_splits_into_two_segments() {
    let start = at(2026, 8, 6, 23, 0, 0);
    let mid = at(2026, 8, 7, 0, 0, 0);
    let end = at(2026, 8, 7, 1, 0, 0);

    let segments = split_by_day(start, end, &Utc, TimeDelta::zero());

    assert_eq!(
        segments,
        vec![
            segment("2026-08-06", start, mid),
            segment("2026-08-07", mid, end),
        ]
    );
}

#[test]
fn interval_crossing_the_04_00_boundary_splits_by_offset_boundary() {
    let start = at(2026, 8, 6, 2, 0, 0);
    let mid = at(2026, 8, 6, 4, 0, 0);
    let end = at(2026, 8, 6, 6, 0, 0);

    let segments = split_by_day(start, end, &Utc, TimeDelta::hours(4));

    assert_eq!(
        segments,
        vec![
            segment("2026-08-05", start, mid),
            segment("2026-08-06", mid, end),
        ]
    );
}

#[test]
fn interval_longer_than_a_day_produces_one_segment_per_day() {
    let start = at(2026, 8, 6, 9, 0, 0);
    let b1 = at(2026, 8, 7, 0, 0, 0);
    let b2 = at(2026, 8, 8, 0, 0, 0);
    let end = at(2026, 8, 8, 9, 0, 0);

    let segments = split_by_day(start, end, &Utc, TimeDelta::zero());

    assert_eq!(
        segments,
        vec![
            segment("2026-08-06", start, b1),
            segment("2026-08-07", b1, b2),
            segment("2026-08-08", b2, end),
        ]
    );
    assert_eq!(segments[0].duration(), TimeDelta::hours(15));
    assert_eq!(segments[1].duration(), TimeDelta::hours(24));
    assert_eq!(segments[2].duration(), TimeDelta::hours(9));
}

#[test]
fn empty_interval_produces_no_segments() {
    let moment = at(2026, 8, 6, 9, 0, 0);
    assert_eq!(
        split_by_day(moment, moment, &Utc, TimeDelta::zero()),
        vec![]
    );
}

#[test]
fn reversed_interval_produces_no_segments() {
    let start = at(2026, 8, 6, 10, 0, 0);
    let end = at(2026, 8, 6, 9, 0, 0);
    assert_eq!(split_by_day(start, end, &Utc, TimeDelta::zero()), vec![]);
}

#[test]
fn interval_ending_exactly_on_a_boundary_has_no_trailing_empty_segment() {
    let start = at(2026, 8, 6, 0, 0, 0);
    let end = at(2026, 8, 7, 0, 0, 0);

    let segments = split_by_day(start, end, &Utc, TimeDelta::zero());

    assert_eq!(segments, vec![segment("2026-08-06", start, end)]);
}

// --- daylight saving transitions -----------------------------------------

#[test]
fn dst_spring_forward_produces_a_23_hour_day() {
    // Offset jumps +1h -> +2h at the transition instant, as in the EU rule.
    let tz = TransitionZone {
        switch_at: at(2026, 3, 29, 1, 0, 0),
        before: FixedOffset::east_opt(3600).unwrap(),
        after: FixedOffset::east_opt(2 * 3600).unwrap(),
    };
    let start = at(2026, 3, 28, 23, 0, 0); // 2026-03-29T00:00 local (+1h)
    let mid = at(2026, 3, 29, 22, 0, 0); // 2026-03-30T00:00 local (+2h)
    let end = at(2026, 3, 30, 22, 0, 0); // 2026-03-31T00:00 local (+2h)

    let segments = split_by_day(start, end, &tz, TimeDelta::zero());

    assert_eq!(
        segments,
        vec![
            segment("2026-03-29", start, mid),
            segment("2026-03-30", mid, end),
        ]
    );
    assert_eq!(segments[0].duration(), TimeDelta::hours(23));
}

#[test]
fn dst_fall_back_produces_a_25_hour_day() {
    // Offset falls +2h -> +1h at the transition instant.
    let tz = TransitionZone {
        switch_at: at(2026, 10, 25, 1, 0, 0),
        before: FixedOffset::east_opt(2 * 3600).unwrap(),
        after: FixedOffset::east_opt(3600).unwrap(),
    };
    let start = at(2026, 10, 24, 22, 0, 0); // 2026-10-25T00:00 local (+2h)
    let mid = at(2026, 10, 25, 23, 0, 0); // 2026-10-26T00:00 local (+1h)
    let end = at(2026, 10, 26, 23, 0, 0); // 2026-10-27T00:00 local (+1h)

    let segments = split_by_day(start, end, &tz, TimeDelta::zero());

    assert_eq!(
        segments,
        vec![
            segment("2026-10-25", start, mid),
            segment("2026-10-26", mid, end),
        ]
    );
    assert_eq!(segments[0].duration(), TimeDelta::hours(25));
}

#[test]
fn day_start_inside_a_dst_gap_does_not_panic_and_still_tiles_the_interval() {
    // day_start (01:30) falls inside the hour the offset jumps across.
    // Which exact UTC instant this resolves to is implementation-defined;
    // what must hold is that `split_by_day` still terminates and produces
    // contiguous, non-overlapping segments covering the whole interval.
    let tz = TransitionZone {
        switch_at: at(2026, 3, 29, 1, 0, 0),
        before: FixedOffset::east_opt(3600).unwrap(),
        after: FixedOffset::east_opt(2 * 3600).unwrap(),
    };
    let start = at(2026, 3, 28, 12, 0, 0);
    let end = at(2026, 3, 30, 12, 0, 0);

    let segments = split_by_day(start, end, &tz, TimeDelta::minutes(90));

    assert!(!segments.is_empty());
    assert_eq!(segments.first().unwrap().start, start);
    assert_eq!(segments.last().unwrap().end, end);
    for pair in segments.windows(2) {
        assert_eq!(pair[0].end, pair[1].start);
    }
    for s in &segments {
        assert!(s.start < s.end);
    }
}
