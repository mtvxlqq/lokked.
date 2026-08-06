//! `Timer` must round-trip through JSON without losing time: the app can be
//! killed by the OS mid-session on mobile, and whatever persisted it has to
//! reconstruct a `Timer` that reports the same `state_at` as the one that
//! was serialised, without the gap while it was gone silently disappearing.

use chrono::{TimeDelta, TimeZone, Utc};
use lokked_lib::core::clock::FakeClock;
use lokked_lib::core::timer::{Mode, Timer};

fn pomodoro() -> Mode {
    Mode::Pomodoro {
        work: TimeDelta::minutes(25),
        short_break: TimeDelta::minutes(5),
        long_break: TimeDelta::minutes(15),
        cycles_before_long_break: 4,
        auto_start_next: true,
    }
}

#[test]
fn a_running_timer_survives_a_serde_round_trip() {
    let clock = FakeClock::new(Utc.with_ymd_and_hms(2026, 8, 6, 9, 0, 0).unwrap());
    let mut timer = Timer::start(Mode::CountUp, &clock);
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
fn a_pomodoro_timer_survives_a_serde_round_trip_mid_break() {
    let clock = FakeClock::new(Utc.with_ymd_and_hms(2026, 8, 6, 9, 0, 0).unwrap());
    let mut timer = Timer::start(pomodoro(), &clock);
    clock.advance(TimeDelta::minutes(25));
    timer.skip_phase(&clock).unwrap(); // into the break
    clock.advance(TimeDelta::minutes(2));
    timer.mark_interruption().unwrap();

    let json = serde_json::to_string(&timer).unwrap();
    let restored: Timer = serde_json::from_str(&json).unwrap();

    assert_eq!(restored, timer);
    assert_eq!(restored.state_at(&clock), timer.state_at(&clock));
    assert_eq!(restored.interruptions(), 1);
}

#[test]
fn a_finished_timer_survives_a_serde_round_trip() {
    let clock = FakeClock::new(Utc.with_ymd_and_hms(2026, 8, 6, 9, 0, 0).unwrap());
    let mut timer = Timer::start(Mode::CountUp, &clock);
    clock.advance(TimeDelta::minutes(45));
    timer.finish(&clock).unwrap();

    let json = serde_json::to_string(&timer).unwrap();
    let restored: Timer = serde_json::from_str(&json).unwrap();

    clock.advance(TimeDelta::hours(2));
    assert_eq!(restored, timer);
    assert_eq!(restored.elapsed(&clock), TimeDelta::minutes(45));
}

#[test]
fn being_gone_for_an_hour_is_recovered_correctly_after_restoring() {
    // Simulates the mobile scenario the whole design is built for: the app
    // starts a countdown, gets killed, and an hour later a fresh process
    // deserialises the same JSON and has to report the right numbers.
    let clock = FakeClock::new(Utc.with_ymd_and_hms(2026, 8, 6, 9, 0, 0).unwrap());
    let timer = Timer::start(
        Mode::CountDown {
            target: TimeDelta::minutes(20),
        },
        &clock,
    );
    let json = serde_json::to_string(&timer).unwrap();
    drop(timer); // the process is gone

    clock.advance(TimeDelta::hours(1));
    let restored: Timer = serde_json::from_str(&json).unwrap();
    let state = restored.state_at(&clock);

    assert_eq!(state.elapsed, TimeDelta::hours(1));
    assert_eq!(state.remaining, Some(TimeDelta::zero()));
    assert!(state.finished);
}

#[test]
fn the_serialised_shape_is_a_flat_object_discriminated_by_status() {
    // The frontend types this as a discriminated union on `status`, and the
    // DB layer will store it, so the shape is a contract, not an
    // implementation detail free to drift.
    let clock = FakeClock::new(Utc.with_ymd_and_hms(2026, 8, 6, 9, 0, 0).unwrap());
    let mut timer = Timer::start(Mode::CountUp, &clock);
    assert_eq!(
        serde_json::to_value(&timer).unwrap(),
        serde_json::json!({
            "mode": { "type": "count_up" },
            "cycle": 1,
            "interruptions": 0,
            "phase": "work",
            "phase_started_at": "2026-08-06T09:00:00Z",
            "pauses": [],
            "status": "running",
        })
    );

    clock.advance(TimeDelta::minutes(10));
    timer.pause(&clock).unwrap();
    assert_eq!(
        serde_json::to_value(&timer).unwrap(),
        serde_json::json!({
            "mode": { "type": "count_up" },
            "cycle": 1,
            "interruptions": 0,
            "phase": "work",
            "phase_started_at": "2026-08-06T09:00:00Z",
            "pauses": [],
            "status": "paused",
            "since": "2026-08-06T09:10:00Z",
        })
    );

    clock.advance(TimeDelta::minutes(5));
    timer.resume(&clock).unwrap();
    timer.finish(&clock).unwrap();
    assert_eq!(
        serde_json::to_value(&timer).unwrap(),
        serde_json::json!({
            "mode": { "type": "count_up" },
            "cycle": 1,
            "interruptions": 0,
            "phase": "work",
            "phase_started_at": "2026-08-06T09:00:00Z",
            "pauses": [{
                "started_at": "2026-08-06T09:10:00Z",
                "ended_at": "2026-08-06T09:15:00Z",
            }],
            "status": "finished",
            "at": "2026-08-06T09:15:00Z",
        })
    );
}

#[test]
fn the_pomodoro_mode_serialises_with_its_own_target_fields() {
    let clock = FakeClock::new(Utc.with_ymd_and_hms(2026, 8, 6, 9, 0, 0).unwrap());
    let timer = Timer::start(pomodoro(), &clock);

    let value = serde_json::to_value(&timer).unwrap();
    assert_eq!(
        value["mode"],
        serde_json::json!({
            "type": "pomodoro",
            "work": [1500, 0],
            "short_break": [300, 0],
            "long_break": [900, 0],
            "cycles_before_long_break": 4,
            "auto_start_next": true,
        })
    );
}

#[test]
fn state_at_serialises_as_a_plain_snapshot() {
    let clock = FakeClock::new(Utc.with_ymd_and_hms(2026, 8, 6, 9, 0, 0).unwrap());
    let timer = Timer::start(
        Mode::CountDown {
            target: TimeDelta::minutes(20),
        },
        &clock,
    );
    clock.advance(TimeDelta::minutes(5));

    assert_eq!(
        serde_json::to_value(timer.state_at(&clock)).unwrap(),
        serde_json::json!({
            "elapsed": [300, 0],
            "remaining": [900, 0],
            "phase": "work",
            "cycle": 1,
            "finished": false,
        })
    );
}
