//! Behavioural tests for the study timer state machine.
//!
//! Kept out of `src/core/timer.rs` so both files stay well under the 400-line
//! limit, and so the tests only ever touch the public API — the same surface
//! `commands.rs` will use.
//!
//! Every test drives a [`FakeClock`]; nothing here sleeps.

use chrono::{DateTime, TimeDelta, TimeZone, Utc};
use lokked_lib::core::clock::{Clock, FakeClock};
use lokked_lib::core::timer::{Timer, TimerError};

/// 2026-08-06 at the given wall-clock time, UTC.
fn at(hour: u32, minute: u32, second: u32) -> DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 8, 6, hour, minute, second)
        .unwrap()
}

fn clock_at(hour: u32, minute: u32) -> FakeClock {
    FakeClock::new(at(hour, minute, 0))
}

// --- starting ------------------------------------------------------------

#[test]
fn a_started_timer_is_running_from_the_current_time() {
    let clock = clock_at(9, 0);

    let timer = Timer::start(&clock);

    assert!(timer.is_running());
    assert!(!timer.is_paused());
    assert!(!timer.is_finished());
    assert_eq!(timer.started_at(), at(9, 0, 0));
    assert_eq!(timer.finished_at(), None);
}

#[test]
fn a_fresh_timer_has_no_elapsed_time() {
    let clock = clock_at(9, 0);

    let timer = Timer::start(&clock);

    assert_eq!(timer.elapsed(&clock), TimeDelta::zero());
    assert_eq!(timer.paused(&clock), TimeDelta::zero());
}

#[test]
fn a_running_timer_elapses_with_the_clock() {
    let clock = clock_at(9, 0);
    let timer = Timer::start(&clock);

    clock.advance(TimeDelta::minutes(25));

    assert_eq!(timer.elapsed(&clock), TimeDelta::minutes(25));
}

// --- pausing and resuming ------------------------------------------------

#[test]
fn a_paused_timer_stops_elapsing() {
    let clock = clock_at(9, 0);
    let mut timer = Timer::start(&clock);
    clock.advance(TimeDelta::minutes(10));

    timer.pause(&clock).unwrap();
    clock.advance(TimeDelta::minutes(5));

    assert!(timer.is_paused());
    assert_eq!(timer.elapsed(&clock), TimeDelta::minutes(10));
}

#[test]
fn time_spent_paused_is_reported_separately() {
    let clock = clock_at(9, 0);
    let mut timer = Timer::start(&clock);
    clock.advance(TimeDelta::minutes(10));
    timer.pause(&clock).unwrap();

    clock.advance(TimeDelta::minutes(5));

    assert_eq!(timer.paused(&clock), TimeDelta::minutes(5));
}

#[test]
fn resuming_continues_from_where_the_timer_stopped() {
    let clock = clock_at(9, 0);
    let mut timer = Timer::start(&clock);
    clock.advance(TimeDelta::minutes(10));
    timer.pause(&clock).unwrap();
    clock.advance(TimeDelta::minutes(5));

    timer.resume(&clock).unwrap();
    clock.advance(TimeDelta::minutes(3));

    assert!(timer.is_running());
    assert_eq!(timer.elapsed(&clock), TimeDelta::minutes(13));
    assert_eq!(timer.paused(&clock), TimeDelta::minutes(5));
}

#[test]
fn several_pauses_all_come_off_the_elapsed_time() {
    let clock = clock_at(9, 0);
    let mut timer = Timer::start(&clock);

    for _ in 0..3 {
        clock.advance(TimeDelta::minutes(20));
        timer.pause(&clock).unwrap();
        clock.advance(TimeDelta::minutes(5));
        timer.resume(&clock).unwrap();
    }

    // 75 wall-clock minutes, 15 of them paused.
    assert_eq!(clock.now(), at(10, 15, 0));
    assert_eq!(timer.elapsed(&clock), TimeDelta::minutes(60));
    assert_eq!(timer.paused(&clock), TimeDelta::minutes(15));
    assert_eq!(timer.pauses().len(), 3);
}

#[test]
fn pausing_a_paused_timer_is_rejected() {
    let clock = clock_at(9, 0);
    let mut timer = Timer::start(&clock);
    timer.pause(&clock).unwrap();

    assert_eq!(timer.pause(&clock), Err(TimerError::AlreadyPaused));
}

#[test]
fn resuming_a_running_timer_is_rejected() {
    let clock = clock_at(9, 0);
    let mut timer = Timer::start(&clock);

    assert_eq!(timer.resume(&clock), Err(TimerError::NotPaused));
}

#[test]
fn a_rejected_transition_leaves_the_timer_untouched() {
    let clock = clock_at(9, 0);
    let mut timer = Timer::start(&clock);
    clock.advance(TimeDelta::minutes(10));
    timer.pause(&clock).unwrap();
    clock.advance(TimeDelta::minutes(5));

    timer.pause(&clock).unwrap_err();

    // Still paused since 9:10, so still 10 minutes of study and 5 of pause.
    assert!(timer.is_paused());
    assert_eq!(timer.elapsed(&clock), TimeDelta::minutes(10));
    assert_eq!(timer.paused(&clock), TimeDelta::minutes(5));
}

// --- finishing -----------------------------------------------------------

#[test]
fn finishing_freezes_the_elapsed_time() {
    let clock = clock_at(9, 0);
    let mut timer = Timer::start(&clock);
    clock.advance(TimeDelta::minutes(45));

    timer.finish(&clock).unwrap();
    clock.advance(TimeDelta::hours(3));

    assert!(timer.is_finished());
    assert_eq!(timer.finished_at(), Some(at(9, 45, 0)));
    assert_eq!(timer.elapsed(&clock), TimeDelta::minutes(45));
}

#[test]
fn finishing_while_paused_closes_the_open_pause() {
    let clock = clock_at(9, 0);
    let mut timer = Timer::start(&clock);
    clock.advance(TimeDelta::minutes(30));
    timer.pause(&clock).unwrap();
    clock.advance(TimeDelta::minutes(10));

    timer.finish(&clock).unwrap();
    clock.advance(TimeDelta::hours(1));

    assert_eq!(timer.elapsed(&clock), TimeDelta::minutes(30));
    assert_eq!(timer.paused(&clock), TimeDelta::minutes(10));
    assert_eq!(timer.pauses().len(), 1);
}

#[test]
fn a_finished_timer_rejects_every_further_transition() {
    let clock = clock_at(9, 0);
    let mut timer = Timer::start(&clock);
    timer.finish(&clock).unwrap();

    assert_eq!(timer.pause(&clock), Err(TimerError::AlreadyFinished));
    assert_eq!(timer.resume(&clock), Err(TimerError::AlreadyFinished));
    assert_eq!(timer.finish(&clock), Err(TimerError::AlreadyFinished));
}

// --- surviving the real world --------------------------------------------

#[test]
fn elapsed_is_computed_from_timestamps_not_from_being_observed() {
    // The app is killed by the OS the moment the timer starts and nothing
    // observes it for two hours. The elapsed time must still be two hours:
    // it is a function of the timestamps, not of how often we looked.
    let clock = clock_at(9, 0);
    let timer = Timer::start(&clock);

    clock.advance(TimeDelta::hours(2));

    assert_eq!(timer.elapsed(&clock), TimeDelta::hours(2));
}

#[test]
fn a_timer_survives_a_serde_round_trip() {
    let clock = clock_at(9, 0);
    let mut timer = Timer::start(&clock);
    clock.advance(TimeDelta::minutes(20));
    timer.pause(&clock).unwrap();
    clock.advance(TimeDelta::minutes(5));
    timer.resume(&clock).unwrap();

    let json = serde_json::to_string(&timer).unwrap();
    let restored: Timer = serde_json::from_str(&json).unwrap();

    clock.advance(TimeDelta::minutes(10));
    assert_eq!(restored, timer);
    assert_eq!(restored.elapsed(&clock), TimeDelta::minutes(30));
}

#[test]
fn the_serialised_shape_is_a_flat_union_discriminated_by_phase() {
    // The frontend types this as a discriminated union on `phase`, and the DB
    // layer will store it, so the shape is a contract — not an implementation
    // detail free to drift.
    let clock = clock_at(9, 0);
    let mut timer = Timer::start(&clock);
    assert_eq!(
        serde_json::to_value(&timer).unwrap(),
        serde_json::json!({
            "started_at": "2026-08-06T09:00:00Z",
            "pauses": [],
            "phase": "running",
        })
    );

    clock.advance(TimeDelta::minutes(10));
    timer.pause(&clock).unwrap();
    assert_eq!(
        serde_json::to_value(&timer).unwrap(),
        serde_json::json!({
            "started_at": "2026-08-06T09:00:00Z",
            "pauses": [],
            "phase": "paused",
            "since": "2026-08-06T09:10:00Z",
        })
    );

    clock.advance(TimeDelta::minutes(5));
    timer.resume(&clock).unwrap();
    timer.finish(&clock).unwrap();
    assert_eq!(
        serde_json::to_value(&timer).unwrap(),
        serde_json::json!({
            "started_at": "2026-08-06T09:00:00Z",
            "pauses": [{
                "started_at": "2026-08-06T09:10:00Z",
                "ended_at": "2026-08-06T09:15:00Z",
            }],
            "phase": "finished",
            "at": "2026-08-06T09:15:00Z",
        })
    );
}

#[test]
fn a_backwards_clock_never_yields_a_negative_elapsed_time() {
    // NTP corrects the machine's clock backwards mid-session.
    let clock = clock_at(9, 0);
    let timer = Timer::start(&clock);

    clock.advance(TimeDelta::minutes(-30));

    assert_eq!(timer.elapsed(&clock), TimeDelta::zero());
}

#[test]
fn a_backwards_clock_never_yields_a_negative_pause() {
    let clock = clock_at(9, 0);
    let mut timer = Timer::start(&clock);
    clock.advance(TimeDelta::minutes(30));
    timer.pause(&clock).unwrap();

    clock.advance(TimeDelta::minutes(-10));
    timer.resume(&clock).unwrap();
    clock.set(at(9, 40, 0));

    assert_eq!(timer.paused(&clock), TimeDelta::zero());
    assert_eq!(timer.elapsed(&clock), TimeDelta::minutes(40));
}

#[test]
fn sub_second_precision_is_preserved() {
    let clock = FakeClock::new(at(9, 0, 0));
    let timer = Timer::start(&clock);

    clock.advance(TimeDelta::milliseconds(1_500));

    assert_eq!(timer.elapsed(&clock), TimeDelta::milliseconds(1_500));
}
