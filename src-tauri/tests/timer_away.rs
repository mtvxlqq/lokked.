//! Tests for `Timer::discard_span`: throwing away wall-clock time the student
//! says they were not studying — the app was in the background, or the
//! machine was asleep.

use chrono::{TimeDelta, TimeZone, Utc};
use lokked_lib::core::clock::{Clock, FakeClock};
use lokked_lib::core::timer::{Mode, Timer, TimerError};

fn start() -> (FakeClock, Timer) {
    let clock = FakeClock::new(Utc.with_ymd_and_hms(2026, 8, 21, 9, 0, 0).unwrap());
    let timer = Timer::start(Mode::CountUp, &clock);
    (clock, timer)
}

#[test]
fn discarded_time_stops_counting_as_studied() {
    let (clock, mut timer) = start();
    let away_from = clock.now() + TimeDelta::minutes(5);
    clock.advance(TimeDelta::minutes(65));

    timer
        .discard_span(away_from, away_from + TimeDelta::minutes(60), &clock)
        .unwrap();

    // 65 minutes passed, an hour of it discarded.
    assert_eq!(timer.elapsed(&clock), TimeDelta::minutes(5));
    assert_eq!(timer.paused(&clock), TimeDelta::minutes(60));
}

#[test]
fn time_before_the_phase_started_cannot_be_discarded() {
    let (clock, mut timer) = start();
    let phase_start = clock.now();
    clock.advance(TimeDelta::minutes(30));

    timer
        .discard_span(
            phase_start - TimeDelta::hours(1),
            phase_start + TimeDelta::minutes(10),
            &clock,
        )
        .unwrap();

    // Only the ten minutes inside the phase are dropped, not the hour before it.
    assert_eq!(timer.elapsed(&clock), TimeDelta::minutes(20));
}

#[test]
fn time_that_has_not_happened_yet_cannot_be_discarded() {
    let (clock, mut timer) = start();
    let from = clock.now() + TimeDelta::minutes(10);
    clock.advance(TimeDelta::minutes(30));

    timer
        .discard_span(from, from + TimeDelta::hours(5), &clock)
        .unwrap();

    assert_eq!(timer.elapsed(&clock), TimeDelta::minutes(10));
}

#[test]
fn an_already_paused_stretch_is_not_discarded_twice() {
    let (clock, mut timer) = start();
    clock.advance(TimeDelta::minutes(10));
    timer.pause(&clock).unwrap();
    clock.advance(TimeDelta::minutes(20));
    timer.resume(&clock).unwrap();
    clock.advance(TimeDelta::minutes(10));

    // 40 minutes of wall clock, 20 of them already paused.
    assert_eq!(timer.elapsed(&clock), TimeDelta::minutes(20));

    let phase_start = clock.now() - TimeDelta::minutes(40);
    timer
        .discard_span(phase_start, clock.now(), &clock)
        .unwrap();

    // Everything is discarded, but the 20 already-paused minutes are not
    // subtracted a second time.
    assert_eq!(timer.elapsed(&clock), TimeDelta::zero());
    assert_eq!(timer.paused(&clock), TimeDelta::minutes(40));
}

#[test]
fn discarding_around_an_open_pause_keeps_the_pause_open() {
    let (clock, mut timer) = start();
    clock.advance(TimeDelta::minutes(10));
    timer.pause(&clock).unwrap();
    clock.advance(TimeDelta::minutes(50));

    let phase_start = clock.now() - TimeDelta::hours(1);
    timer
        .discard_span(phase_start, clock.now(), &clock)
        .unwrap();

    assert!(timer.is_paused());
    assert_eq!(timer.elapsed(&clock), TimeDelta::zero());

    // The pause is still open, so resuming later still works.
    clock.advance(TimeDelta::minutes(5));
    timer.resume(&clock).unwrap();
    assert_eq!(timer.elapsed(&clock), TimeDelta::zero());
}

#[test]
fn an_empty_or_backwards_span_changes_nothing() {
    let (clock, mut timer) = start();
    clock.advance(TimeDelta::minutes(30));
    let before = timer.clone();

    timer
        .discard_span(clock.now(), clock.now(), &clock)
        .unwrap();
    timer
        .discard_span(clock.now(), clock.now() - TimeDelta::hours(1), &clock)
        .unwrap();

    assert_eq!(timer, before);
}

#[test]
fn a_finished_session_cannot_be_edited() {
    let (clock, mut timer) = start();
    clock.advance(TimeDelta::minutes(30));
    timer.finish(&clock).unwrap();

    assert_eq!(
        timer.discard_span(clock.now() - TimeDelta::hours(1), clock.now(), &clock),
        Err(TimerError::AlreadyFinished)
    );
}

#[test]
fn a_discarded_span_survives_a_serde_round_trip() {
    let (clock, mut timer) = start();
    clock.advance(TimeDelta::minutes(65));
    timer
        .discard_span(clock.now() - TimeDelta::minutes(60), clock.now(), &clock)
        .unwrap();

    let json = serde_json::to_string(&timer).unwrap();
    let restored: Timer = serde_json::from_str(&json).unwrap();

    assert_eq!(restored, timer);
    assert_eq!(restored.elapsed(&clock), TimeDelta::minutes(5));
}

// --- паузы при выходе из фазы -------------------------------------------

#[test]
fn pauses_at_closes_an_open_pause_at_the_current_time() {
    let (clock, mut timer) = start();
    clock.advance(TimeDelta::minutes(10));
    timer.pause(&clock).unwrap();
    let paused_at = clock.now();
    clock.advance(TimeDelta::minutes(5));

    let pauses = timer.pauses_at(&clock);

    // The stored state still has no closed pause — only the view does.
    assert!(timer.pauses().is_empty());
    assert_eq!(pauses.len(), 1);
    assert_eq!(pauses[0].started_at, paused_at);
    assert_eq!(pauses[0].ended_at, clock.now());
}

#[test]
fn pauses_at_matches_the_stored_pauses_while_running() {
    let (clock, mut timer) = start();
    clock.advance(TimeDelta::minutes(10));
    timer.pause(&clock).unwrap();
    clock.advance(TimeDelta::minutes(5));
    timer.resume(&clock).unwrap();

    assert_eq!(timer.pauses_at(&clock), timer.pauses().to_vec());
}
