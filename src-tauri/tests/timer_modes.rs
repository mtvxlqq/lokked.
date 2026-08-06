//! Tests for how `Mode` shapes a `Timer`: targets, `remaining`/`finished`,
//! and — for `Pomodoro` — moving between phases with `skip_phase`.
//!
//! Lifecycle behaviour that is the same across every mode (pause/resume/
//! finish/interruptions) lives in `tests/timer_lifecycle.rs`.

use chrono::{DateTime, TimeDelta, TimeZone, Utc};
use lokked_lib::core::clock::FakeClock;
use lokked_lib::core::timer::{Mode, SessionPhase, Timer, TimerError};

fn at(hour: u32, minute: u32, second: u32) -> DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 8, 6, hour, minute, second)
        .unwrap()
}

fn clock_at(hour: u32, minute: u32) -> FakeClock {
    FakeClock::new(at(hour, minute, 0))
}

fn pomodoro(cycles_before_long_break: u32, auto_start_next: bool) -> Mode {
    Mode::Pomodoro {
        work: TimeDelta::minutes(25),
        short_break: TimeDelta::minutes(5),
        long_break: TimeDelta::minutes(15),
        cycles_before_long_break,
        auto_start_next,
    }
}

// --- CountUp: a stopwatch with no target ------------------------------------

#[test]
fn count_up_never_reports_a_target_or_finishes() {
    let clock = clock_at(9, 0);
    let timer = Timer::start(Mode::CountUp, &clock);

    clock.advance(TimeDelta::hours(3));
    let state = timer.state_at(&clock);

    assert_eq!(state.remaining, None);
    assert!(!state.finished);
    assert_eq!(state.phase, SessionPhase::Work);
    assert_eq!(state.cycle, 1);
}

#[test]
fn count_up_rejects_skip_phase() {
    let clock = clock_at(9, 0);
    let mut timer = Timer::start(Mode::CountUp, &clock);

    assert_eq!(timer.skip_phase(&clock), Err(TimerError::NoNextPhase));
}

// --- CountDown: a target that does not stop the timer -----------------------

#[test]
fn count_down_reports_remaining_time_until_the_target() {
    let clock = clock_at(9, 0);
    let timer = Timer::start(
        Mode::CountDown {
            target: TimeDelta::minutes(20),
        },
        &clock,
    );

    clock.advance(TimeDelta::minutes(5));

    let state = timer.state_at(&clock);
    assert_eq!(state.remaining, Some(TimeDelta::minutes(15)));
    assert!(!state.finished);
}

#[test]
fn count_down_reports_finished_once_the_target_is_reached() {
    let clock = clock_at(9, 0);
    let timer = Timer::start(
        Mode::CountDown {
            target: TimeDelta::minutes(20),
        },
        &clock,
    );

    clock.advance(TimeDelta::minutes(20));

    let state = timer.state_at(&clock);
    assert_eq!(state.remaining, Some(TimeDelta::zero()));
    assert!(state.finished);
}

#[test]
fn count_down_keeps_running_past_the_target_instead_of_stopping() {
    // Reaching zero flips `finished` in the snapshot; it does not end the
    // session. The app is gone for an hour past the target — elapsed keeps
    // growing, `remaining` stays clamped at zero, and the timer is still
    // running until something explicitly calls `finish`.
    let clock = clock_at(9, 0);
    let timer = Timer::start(
        Mode::CountDown {
            target: TimeDelta::minutes(20),
        },
        &clock,
    );

    clock.advance(TimeDelta::hours(1) + TimeDelta::minutes(20));

    assert!(timer.is_running());
    let state = timer.state_at(&clock);
    assert_eq!(state.elapsed, TimeDelta::hours(1) + TimeDelta::minutes(20));
    assert_eq!(state.remaining, Some(TimeDelta::zero()));
    assert!(state.finished);
}

#[test]
fn count_down_rejects_skip_phase() {
    let clock = clock_at(9, 0);
    let mut timer = Timer::start(
        Mode::CountDown {
            target: TimeDelta::minutes(20),
        },
        &clock,
    );

    assert_eq!(timer.skip_phase(&clock), Err(TimerError::NoNextPhase));
}

// --- Pomodoro: cycling through phases ---------------------------------------

#[test]
fn pomodoro_starts_in_the_work_phase_at_cycle_one() {
    let clock = clock_at(9, 0);
    let timer = Timer::start(pomodoro(4, false), &clock);

    assert_eq!(timer.phase(), SessionPhase::Work);
    let state = timer.state_at(&clock);
    assert_eq!(state.phase, SessionPhase::Work);
    assert_eq!(state.cycle, 1);
    assert_eq!(state.remaining, Some(TimeDelta::minutes(25)));
}

#[test]
fn a_full_pomodoro_cycle_takes_a_long_break_after_the_fourth_work_phase() {
    // 25/5 work/break, long break after the 4th work phase — the scenario
    // PLAN.md calls out explicitly for this milestone.
    let clock = clock_at(9, 0);
    let mut timer = Timer::start(pomodoro(4, false), &clock);

    let expect = |timer: &Timer, phase: SessionPhase, cycle: u32| {
        let state = timer.state_at(&clock);
        assert_eq!(state.phase, phase, "phase");
        assert_eq!(state.cycle, cycle, "cycle");
    };

    expect(&timer, SessionPhase::Work, 1);
    for cycle in 1..4 {
        timer.skip_phase(&clock).unwrap(); // end work
        expect(&timer, SessionPhase::Break, cycle);
        timer.skip_phase(&clock).unwrap(); // end break
        expect(&timer, SessionPhase::Work, cycle + 1);
    }

    // Now on the 4th work phase.
    expect(&timer, SessionPhase::Work, 4);
    timer.skip_phase(&clock).unwrap(); // end 4th work -> long break, not a short one
    expect(&timer, SessionPhase::LongBreak, 4);

    timer.skip_phase(&clock).unwrap(); // end long break -> back to work, cycle resets
    expect(&timer, SessionPhase::Work, 1);
}

#[test]
fn skip_phase_ignores_how_much_of_the_target_has_elapsed() {
    // Skipping is a forced transition, not conditional on the phase timer
    // running out — the user can bail out of a work phase early.
    let clock = clock_at(9, 0);
    let mut timer = Timer::start(pomodoro(4, false), &clock);
    clock.advance(TimeDelta::minutes(2)); // nowhere near the 25-minute target

    timer.skip_phase(&clock).unwrap();

    assert_eq!(timer.phase(), SessionPhase::Break);
}

#[test]
fn skip_phase_starts_the_new_phase_fresh() {
    let clock = clock_at(9, 0);
    let mut timer = Timer::start(pomodoro(4, false), &clock);
    clock.advance(TimeDelta::minutes(10));
    timer.mark_interruption().unwrap();

    timer.skip_phase(&clock).unwrap();

    assert_eq!(timer.phase_started_at(), at(9, 10, 0));
    assert_eq!(timer.elapsed(&clock), TimeDelta::zero());
    assert_eq!(timer.interruptions(), 0);
    assert!(timer.pauses().is_empty());
}

#[test]
fn skip_phase_while_paused_closes_the_pause_and_leaves_the_new_phase_running() {
    let clock = clock_at(9, 0);
    let mut timer = Timer::start(pomodoro(4, false), &clock);
    clock.advance(TimeDelta::minutes(10));
    timer.pause(&clock).unwrap();
    clock.advance(TimeDelta::minutes(3));

    timer.skip_phase(&clock).unwrap();

    assert!(timer.is_running());
    assert_eq!(timer.phase(), SessionPhase::Break);
    // The pause that was open when we skipped belonged to the phase that
    // just ended, so the new phase starts with a clean pause list.
    assert!(timer.pauses().is_empty());
}

#[test]
fn skip_phase_on_a_finished_timer_is_rejected() {
    let clock = clock_at(9, 0);
    let mut timer = Timer::start(pomodoro(4, false), &clock);
    timer.finish(&clock).unwrap();

    assert_eq!(timer.skip_phase(&clock), Err(TimerError::AlreadyFinished));
}

#[test]
fn a_single_work_phase_pomodoro_takes_a_long_break_every_time() {
    // cycles_before_long_break = 1 is a degenerate but legal configuration:
    // every work phase is immediately followed by a long break.
    let clock = clock_at(9, 0);
    let mut timer = Timer::start(pomodoro(1, false), &clock);

    timer.skip_phase(&clock).unwrap();
    assert_eq!(timer.phase(), SessionPhase::LongBreak);

    timer.skip_phase(&clock).unwrap();
    assert_eq!(timer.phase(), SessionPhase::Work);
    assert_eq!(timer.state_at(&clock).cycle, 1);
}

#[test]
fn auto_start_next_reflects_the_configured_flag() {
    let clock = clock_at(9, 0);
    let auto = Timer::start(pomodoro(4, true), &clock);
    let manual = Timer::start(pomodoro(4, false), &clock);
    let count_up = Timer::start(Mode::CountUp, &clock);

    assert!(auto.auto_start_next());
    assert!(!manual.auto_start_next());
    assert!(!count_up.auto_start_next());
}

#[test]
fn pomodoro_break_reports_finished_once_its_own_target_is_reached() {
    let clock = clock_at(9, 0);
    let mut timer = Timer::start(pomodoro(4, false), &clock);
    timer.skip_phase(&clock).unwrap(); // into the 5-minute break

    clock.advance(TimeDelta::minutes(5));

    let state = timer.state_at(&clock);
    assert_eq!(state.phase, SessionPhase::Break);
    assert_eq!(state.remaining, Some(TimeDelta::zero()));
    assert!(state.finished);
}
