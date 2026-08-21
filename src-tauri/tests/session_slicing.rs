//! Tests for `core::session`: turning one finished phase into the rows that
//! go into `sessions`, and deciding whether a return from the background
//! needs to ask the student anything.

use chrono::{DateTime, FixedOffset, TimeDelta, TimeZone, Utc};
use lokked_lib::core::session::{away_report, slice_phase, AWAY_PROMPT_SECONDS};
use lokked_lib::core::timer::Pause;

fn at(y: i32, m: u32, d: u32, h: u32, mi: u32) -> DateTime<Utc> {
    Utc.with_ymd_and_hms(y, m, d, h, mi, 0).unwrap()
}

fn pause(from: DateTime<Utc>, to: DateTime<Utc>) -> Pause {
    Pause {
        started_at: from,
        ended_at: to,
    }
}

/// UTC+3, so a study day boundary lands at 21:00 UTC.
fn moscow() -> FixedOffset {
    FixedOffset::east_opt(3 * 3600).unwrap()
}

#[test]
fn a_phase_inside_one_day_is_a_single_row() {
    let started = at(2026, 8, 21, 9, 0);
    let ended = at(2026, 8, 21, 9, 25);

    let slices = slice_phase(started, ended, &[], &moscow(), TimeDelta::zero());

    assert_eq!(slices.len(), 1);
    assert_eq!(slices[0].day_key, "2026-08-21");
    assert_eq!(slices[0].started_at, started);
    assert_eq!(slices[0].ended_at, ended);
    assert_eq!(slices[0].active_seconds, 25 * 60);
    assert_eq!(slices[0].paused_seconds, 0);
}

#[test]
fn pauses_are_subtracted_from_active_time() {
    let started = at(2026, 8, 21, 9, 0);
    let ended = at(2026, 8, 21, 9, 30);
    let pauses = [pause(at(2026, 8, 21, 9, 10), at(2026, 8, 21, 9, 15))];

    let slices = slice_phase(started, ended, &pauses, &moscow(), TimeDelta::zero());

    assert_eq!(slices[0].active_seconds, 25 * 60);
    assert_eq!(slices[0].paused_seconds, 5 * 60);
}

#[test]
fn an_empty_phase_produces_nothing_to_store() {
    let moment = at(2026, 8, 21, 9, 0);

    assert!(slice_phase(moment, moment, &[], &moscow(), TimeDelta::zero()).is_empty());
    // A clock that jumped backwards must not produce a negative row either.
    let earlier = at(2026, 8, 21, 8, 0);
    assert!(slice_phase(moment, earlier, &[], &moscow(), TimeDelta::zero()).is_empty());
}

#[test]
fn a_phase_crossing_midnight_becomes_one_row_per_day() {
    // 23:30 to 00:30 local (UTC+3) = 20:30 to 21:30 UTC.
    let started = at(2026, 8, 21, 20, 30);
    let ended = at(2026, 8, 21, 21, 30);

    let slices = slice_phase(started, ended, &[], &moscow(), TimeDelta::zero());

    assert_eq!(slices.len(), 2);
    assert_eq!(slices[0].day_key, "2026-08-21");
    assert_eq!(slices[0].active_seconds, 30 * 60);
    assert_eq!(slices[1].day_key, "2026-08-22");
    assert_eq!(slices[1].active_seconds, 30 * 60);
    // The rows meet exactly at the boundary: no second is in both or neither.
    assert_eq!(slices[0].ended_at, slices[1].started_at);
}

#[test]
fn a_pause_is_charged_to_the_day_it_happened_in() {
    let started = at(2026, 8, 21, 20, 30);
    let ended = at(2026, 8, 21, 21, 30);
    // Ten minutes of pause, entirely after the boundary.
    let pauses = [pause(at(2026, 8, 21, 21, 5), at(2026, 8, 21, 21, 15))];

    let slices = slice_phase(started, ended, &pauses, &moscow(), TimeDelta::zero());

    assert_eq!(slices[0].paused_seconds, 0);
    assert_eq!(slices[0].active_seconds, 30 * 60);
    assert_eq!(slices[1].paused_seconds, 10 * 60);
    assert_eq!(slices[1].active_seconds, 20 * 60);
}

#[test]
fn a_pause_spanning_the_boundary_is_split_between_the_days() {
    let started = at(2026, 8, 21, 20, 30);
    let ended = at(2026, 8, 21, 21, 30);
    let pauses = [pause(at(2026, 8, 21, 20, 50), at(2026, 8, 21, 21, 10))];

    let slices = slice_phase(started, ended, &pauses, &moscow(), TimeDelta::zero());

    assert_eq!(slices[0].paused_seconds, 10 * 60);
    assert_eq!(slices[1].paused_seconds, 10 * 60);
    assert_eq!(slices[0].active_seconds, 20 * 60);
    assert_eq!(slices[1].active_seconds, 20 * 60);
}

#[test]
fn a_day_spent_entirely_on_pause_still_produces_a_row() {
    let started = at(2026, 8, 21, 20, 30);
    let ended = at(2026, 8, 21, 21, 30);
    // Paused across the whole first half.
    let pauses = [pause(at(2026, 8, 21, 20, 30), at(2026, 8, 21, 21, 0))];

    let slices = slice_phase(started, ended, &pauses, &moscow(), TimeDelta::zero());

    // The row is kept even at zero active seconds: the session did span that
    // day, and dropping it would lose the pause from the statistics.
    assert_eq!(slices.len(), 2);
    assert_eq!(slices[0].active_seconds, 0);
    assert_eq!(slices[0].paused_seconds, 30 * 60);
}

#[test]
fn pauses_outside_the_phase_are_ignored() {
    let started = at(2026, 8, 21, 9, 0);
    let ended = at(2026, 8, 21, 9, 30);
    let pauses = [
        pause(at(2026, 8, 21, 8, 0), at(2026, 8, 21, 8, 30)),
        pause(at(2026, 8, 21, 10, 0), at(2026, 8, 21, 10, 30)),
    ];

    let slices = slice_phase(started, ended, &pauses, &moscow(), TimeDelta::zero());

    assert_eq!(slices[0].active_seconds, 30 * 60);
    assert_eq!(slices[0].paused_seconds, 0);
}

#[test]
fn a_later_day_start_keeps_a_late_night_phase_in_one_day() {
    // 01:00 to 02:00 local, with the study day starting at 04:00 — still
    // yesterday, and still a single row.
    let started = at(2026, 8, 21, 22, 0);
    let ended = at(2026, 8, 21, 23, 0);

    let slices = slice_phase(started, ended, &[], &moscow(), TimeDelta::hours(4));

    assert_eq!(slices.len(), 1);
    assert_eq!(slices[0].day_key, "2026-08-21");
}

// --- возвращение из фона -----------------------------------------------

#[test]
fn a_short_absence_needs_no_decision() {
    let last_seen = at(2026, 8, 21, 9, 0);
    let now = last_seen + TimeDelta::seconds(AWAY_PROMPT_SECONDS - 1);

    let report = away_report(last_seen, now);

    assert_eq!(report.away_seconds, AWAY_PROMPT_SECONDS - 1);
    assert!(!report.needs_decision);
}

#[test]
fn an_absence_at_the_threshold_asks_the_student() {
    let last_seen = at(2026, 8, 21, 9, 0);
    let now = last_seen + TimeDelta::seconds(AWAY_PROMPT_SECONDS);

    assert!(away_report(last_seen, now).needs_decision);
}

#[test]
fn a_clock_that_went_backwards_is_not_an_absence() {
    let last_seen = at(2026, 8, 21, 9, 0);
    let now = at(2026, 8, 21, 8, 0);

    let report = away_report(last_seen, now);

    assert_eq!(report.away_seconds, 0);
    assert!(!report.needs_decision);
}
