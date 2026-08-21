//! Tests for the session commands: what «Старт» picks, what gets written to
//! `sessions` and when, and what happens to time the app spent away.

use chrono::{TimeDelta, TimeZone, Utc};
use lokked_lib::commands::presets::{self, PresetInput};
use lokked_lib::commands::session::actions::{
    current, discard_away, mark_interruption, pause, report_return, resume, skip_phase, start, stop,
};
use lokked_lib::commands::session::{work_in_progress, SessionState};
use lokked_lib::commands::settings::write_day;
use lokked_lib::commands::subjects::{self, SubjectInput};
use lokked_lib::commands::ErrorKind;
use lokked_lib::core::clock::{Clock, FakeClock};
use lokked_lib::core::dayline::day_key;
use lokked_lib::db::sessions::SessionRepo;
use lokked_lib::db::Database;
use lokked_lib::platform::noop::NoopPlatform;
use lokked_lib::platform::SharedPlatform;

/// Everything a session command needs, wired to fakes.
struct Env {
    db: Database,
    state: SessionState,
    platform: SharedPlatform,
    clock: FakeClock,
}

fn env() -> Env {
    Env {
        db: Database::open_in_memory().expect("in-memory database should open"),
        state: SessionState::default(),
        platform: SharedPlatform::new(Box::new(NoopPlatform)),
        clock: FakeClock::new(Utc.with_ymd_and_hms(2026, 8, 21, 9, 0, 0).unwrap()),
    }
}

fn subject(env: &Env, name: &str) -> String {
    subjects::create(
        &env.db,
        SubjectInput {
            name: name.to_string(),
            color: None,
            icon: None,
        },
    )
    .unwrap()
    .id
}

fn preset(env: &Env, input: PresetInput) -> String {
    presets::create(&env.db, input).unwrap().id
}

fn pomodoro(name: &str) -> PresetInput {
    PresetInput {
        subject_id: None,
        name: name.to_string(),
        mode: "pomodoro".to_string(),
        work_seconds: 25 * 60,
        break_seconds: Some(5 * 60),
        long_break_seconds: Some(15 * 60),
        cycles_before_long: Some(4),
        auto_start_next: false,
        is_default: false,
    }
}

/// Rows in `sessions` for one study day, oldest first.
fn rows(env: &Env, day: &str) -> Vec<lokked_lib::db::sessions::Session> {
    SessionRepo::new(&env.db).list_for_day(day).unwrap()
}

#[test]
fn without_any_preset_a_session_is_a_plain_stopwatch() {
    let env = env();
    let algebra = subject(&env, "Алгебра");

    let view = start(&env.db, &env.state, &env.platform, &env.clock, &algebra).unwrap();

    assert_eq!(view.mode, "countup");
    assert_eq!(view.phase, "work");
    assert_eq!(view.status, "running");
    assert_eq!(view.preset_id, None);
    assert_eq!(view.remaining_seconds, None);
    assert_eq!(view.subject_name, "Алгебра");
}

#[test]
fn the_preset_that_applies_to_the_subject_is_used() {
    let env = env();
    let algebra = subject(&env, "Алгебра");
    preset(&env, pomodoro("Глобальный"));
    let mine = preset(
        &env,
        PresetInput {
            subject_id: Some(algebra.clone()),
            ..pomodoro("Для алгебры")
        },
    );

    let view = start(&env.db, &env.state, &env.platform, &env.clock, &algebra).unwrap();

    assert_eq!(view.preset_id.as_deref(), Some(mine.as_str()));
    assert_eq!(view.mode, "pomodoro");
    assert_eq!(view.target_seconds, Some(25 * 60));
    assert_eq!(view.cycle, 1);
    assert_eq!(view.cycles_before_long, Some(4));
}

#[test]
fn a_second_session_cannot_be_started_over_a_running_one() {
    let env = env();
    let algebra = subject(&env, "Алгебра");
    start(&env.db, &env.state, &env.platform, &env.clock, &algebra).unwrap();

    let error = start(&env.db, &env.state, &env.platform, &env.clock, &algebra).unwrap_err();

    assert_eq!(error.kind, ErrorKind::Conflict);
}

#[test]
fn a_deleted_subject_cannot_be_studied() {
    let env = env();
    let algebra = subject(&env, "Алгебра");
    subjects::delete(&env.db, &algebra).unwrap();

    let error = start(&env.db, &env.state, &env.platform, &env.clock, &algebra).unwrap_err();

    assert_eq!(error.kind, ErrorKind::NotFound);
}

#[test]
fn with_no_session_there_is_nothing_to_show_or_stop() {
    let env = env();

    assert_eq!(
        current(&env.db, &env.state, &env.platform, &env.clock).unwrap(),
        None
    );
    assert_eq!(
        stop(&env.db, &env.state, &env.platform, &env.clock)
            .unwrap_err()
            .kind,
        ErrorKind::Conflict
    );
    assert_eq!(
        pause(&env.state, &env.platform, &env.clock)
            .unwrap_err()
            .kind,
        ErrorKind::Conflict
    );
}

#[test]
fn elapsed_time_comes_from_the_clock_not_from_ticks() {
    let env = env();
    let algebra = subject(&env, "Алгебра");
    start(&env.db, &env.state, &env.platform, &env.clock, &algebra).unwrap();

    // Nothing polls the timer in between; the answer is still right.
    env.clock.advance(TimeDelta::minutes(42));
    let view = current(&env.db, &env.state, &env.platform, &env.clock)
        .unwrap()
        .unwrap();

    assert_eq!(view.elapsed_seconds, 42 * 60);
}

#[test]
fn a_pause_stops_time_from_accruing() {
    let env = env();
    let algebra = subject(&env, "Алгебра");
    start(&env.db, &env.state, &env.platform, &env.clock, &algebra).unwrap();
    env.clock.advance(TimeDelta::minutes(10));

    let paused = pause(&env.state, &env.platform, &env.clock).unwrap();
    assert_eq!(paused.status, "paused");

    env.clock.advance(TimeDelta::minutes(30));
    let view = current(&env.db, &env.state, &env.platform, &env.clock)
        .unwrap()
        .unwrap();
    assert_eq!(view.elapsed_seconds, 10 * 60);

    resume(&env.state, &env.platform, &env.clock).unwrap();
    env.clock.advance(TimeDelta::minutes(5));
    let view = current(&env.db, &env.state, &env.platform, &env.clock)
        .unwrap()
        .unwrap();
    assert_eq!(view.elapsed_seconds, 15 * 60);
    assert_eq!(view.status, "running");
}

#[test]
fn pausing_twice_is_refused_and_changes_nothing() {
    let env = env();
    let algebra = subject(&env, "Алгебра");
    start(&env.db, &env.state, &env.platform, &env.clock, &algebra).unwrap();
    pause(&env.state, &env.platform, &env.clock).unwrap();

    assert_eq!(
        pause(&env.state, &env.platform, &env.clock)
            .unwrap_err()
            .kind,
        ErrorKind::Conflict
    );
    assert_eq!(
        current(&env.db, &env.state, &env.platform, &env.clock)
            .unwrap()
            .unwrap()
            .status,
        "paused"
    );
}

#[test]
fn an_interruption_is_counted_without_stopping_the_clock() {
    let env = env();
    let algebra = subject(&env, "Алгебра");
    start(&env.db, &env.state, &env.platform, &env.clock, &algebra).unwrap();
    env.clock.advance(TimeDelta::minutes(10));

    let view = mark_interruption(&env.state, &env.clock).unwrap();

    assert_eq!(view.interruptions, 1);
    assert_eq!(view.elapsed_seconds, 10 * 60);
    assert_eq!(view.status, "running");
}

#[test]
fn stopping_writes_the_phase_and_clears_the_session() {
    let env = env();
    let algebra = subject(&env, "Алгебра");
    start(&env.db, &env.state, &env.platform, &env.clock, &algebra).unwrap();
    env.clock.advance(TimeDelta::minutes(30));
    mark_interruption(&env.state, &env.clock).unwrap();

    stop(&env.db, &env.state, &env.platform, &env.clock).unwrap();

    let stored = rows(&env, "2026-08-21");
    assert_eq!(stored.len(), 1);
    assert_eq!(stored[0].subject_id, algebra);
    assert_eq!(stored[0].phase, "work");
    assert_eq!(stored[0].mode, "countup");
    assert_eq!(stored[0].active_seconds, 30 * 60);
    assert_eq!(stored[0].paused_seconds, 0);
    assert_eq!(stored[0].interruptions, 1);
    assert!(!stored[0].completed);

    assert_eq!(
        current(&env.db, &env.state, &env.platform, &env.clock).unwrap(),
        None
    );
}

#[test]
fn a_paused_stretch_is_recorded_as_paused_not_studied() {
    let env = env();
    let algebra = subject(&env, "Алгебра");
    start(&env.db, &env.state, &env.platform, &env.clock, &algebra).unwrap();
    env.clock.advance(TimeDelta::minutes(20));
    pause(&env.state, &env.platform, &env.clock).unwrap();
    env.clock.advance(TimeDelta::minutes(15));

    // Stopping while paused still closes the books correctly.
    stop(&env.db, &env.state, &env.platform, &env.clock).unwrap();

    let stored = rows(&env, "2026-08-21");
    assert_eq!(stored[0].active_seconds, 20 * 60);
    assert_eq!(stored[0].paused_seconds, 15 * 60);
}

#[test]
fn a_countdown_that_reached_its_target_is_recorded_as_completed() {
    let env = env();
    let algebra = subject(&env, "Алгебра");
    preset(
        &env,
        PresetInput {
            mode: "countdown".to_string(),
            work_seconds: 45 * 60,
            is_default: true,
            ..pomodoro("45 минут")
        },
    );
    start(&env.db, &env.state, &env.platform, &env.clock, &algebra).unwrap();

    env.clock.advance(TimeDelta::minutes(45));
    let view = current(&env.db, &env.state, &env.platform, &env.clock)
        .unwrap()
        .unwrap();
    assert!(view.phase_finished);
    assert_eq!(view.remaining_seconds, Some(0));

    stop(&env.db, &env.state, &env.platform, &env.clock).unwrap();

    let stored = rows(&env, "2026-08-21");
    assert!(stored[0].completed);
    assert_eq!(stored[0].planned_seconds, Some(45 * 60));
}

#[test]
fn a_countdown_keeps_running_past_its_target() {
    let env = env();
    let algebra = subject(&env, "Алгебра");
    preset(
        &env,
        PresetInput {
            mode: "countdown".to_string(),
            work_seconds: 10 * 60,
            is_default: true,
            ..pomodoro("10 минут")
        },
    );
    start(&env.db, &env.state, &env.platform, &env.clock, &algebra).unwrap();
    env.clock.advance(TimeDelta::minutes(15));

    let view = current(&env.db, &env.state, &env.platform, &env.clock)
        .unwrap()
        .unwrap();

    assert_eq!(view.elapsed_seconds, 15 * 60);
    assert_eq!(view.remaining_seconds, Some(0));
}

#[test]
fn skipping_a_pomodoro_phase_writes_it_down_and_moves_on() {
    let env = env();
    let algebra = subject(&env, "Алгебра");
    preset(
        &env,
        PresetInput {
            is_default: true,
            ..pomodoro("Классический")
        },
    );
    start(&env.db, &env.state, &env.platform, &env.clock, &algebra).unwrap();
    env.clock.advance(TimeDelta::minutes(25));

    let view = skip_phase(&env.db, &env.state, &env.platform, &env.clock).unwrap();

    assert_eq!(view.phase, "break");
    assert_eq!(view.cycle, 1);
    assert_eq!(view.elapsed_seconds, 0);
    assert_eq!(view.target_seconds, Some(5 * 60));

    let stored = rows(&env, "2026-08-21");
    assert_eq!(stored.len(), 1);
    assert_eq!(stored[0].phase, "work");
    assert_eq!(stored[0].active_seconds, 25 * 60);
    assert!(stored[0].completed);
}

#[test]
fn a_stopwatch_has_no_next_phase_to_skip_to() {
    let env = env();
    let algebra = subject(&env, "Алгебра");
    start(&env.db, &env.state, &env.platform, &env.clock, &algebra).unwrap();

    assert_eq!(
        skip_phase(&env.db, &env.state, &env.platform, &env.clock)
            .unwrap_err()
            .kind,
        ErrorKind::Conflict
    );
}

#[test]
fn auto_start_next_rolls_the_phase_over_on_its_own() {
    let env = env();
    let algebra = subject(&env, "Алгебра");
    preset(
        &env,
        PresetInput {
            auto_start_next: true,
            is_default: true,
            ..pomodoro("Автоматический")
        },
    );
    start(&env.db, &env.state, &env.platform, &env.clock, &algebra).unwrap();
    env.clock.advance(TimeDelta::minutes(25));

    // Nothing was pressed: polling the session is what moves it on.
    let view = current(&env.db, &env.state, &env.platform, &env.clock)
        .unwrap()
        .unwrap();

    assert_eq!(view.phase, "break");
    assert_eq!(rows(&env, "2026-08-21").len(), 1);
}

#[test]
fn without_auto_start_a_finished_phase_waits_for_the_student() {
    let env = env();
    let algebra = subject(&env, "Алгебра");
    preset(
        &env,
        PresetInput {
            is_default: true,
            ..pomodoro("Классический")
        },
    );
    start(&env.db, &env.state, &env.platform, &env.clock, &algebra).unwrap();
    env.clock.advance(TimeDelta::minutes(30));

    let view = current(&env.db, &env.state, &env.platform, &env.clock)
        .unwrap()
        .unwrap();

    assert_eq!(view.phase, "work");
    assert!(view.phase_finished);
    assert!(rows(&env, "2026-08-21").is_empty());
}

#[test]
fn a_long_break_comes_after_the_last_cycle() {
    let env = env();
    let algebra = subject(&env, "Алгебра");
    preset(
        &env,
        PresetInput {
            cycles_before_long: Some(2),
            is_default: true,
            ..pomodoro("Короткий круг")
        },
    );
    start(&env.db, &env.state, &env.platform, &env.clock, &algebra).unwrap();

    // work 1 → break → work 2 → long break
    for _ in 0..3 {
        env.clock.advance(TimeDelta::minutes(5));
        skip_phase(&env.db, &env.state, &env.platform, &env.clock).unwrap();
    }
    let view = current(&env.db, &env.state, &env.platform, &env.clock)
        .unwrap()
        .unwrap();

    assert_eq!(view.phase, "long_break");
    assert_eq!(view.cycle, 2);
    assert_eq!(view.target_seconds, Some(15 * 60));
}

#[test]
fn a_session_crossing_midnight_is_stored_as_one_row_per_day() {
    let env = env();
    let algebra = subject(&env, "Алгебра");
    // Start an hour before local midnight and run two hours.
    let local_midnight = chrono::Local
        .with_ymd_and_hms(2026, 8, 22, 0, 0, 0)
        .single()
        .expect("midnight on 22 August is unambiguous");
    env.clock
        .set(local_midnight.with_timezone(&Utc) - TimeDelta::hours(1));
    start(&env.db, &env.state, &env.platform, &env.clock, &algebra).unwrap();

    env.clock.advance(TimeDelta::hours(2));
    stop(&env.db, &env.state, &env.platform, &env.clock).unwrap();

    let before = rows(&env, "2026-08-21");
    let after = rows(&env, "2026-08-22");
    assert_eq!(before.len(), 1);
    assert_eq!(after.len(), 1);
    assert_eq!(before[0].active_seconds, 3600);
    assert_eq!(after[0].active_seconds, 3600);
}

// --- возвращение из фона -------------------------------------------------

#[test]
fn a_short_absence_asks_nothing() {
    let env = env();
    let algebra = subject(&env, "Алгебра");
    start(&env.db, &env.state, &env.platform, &env.clock, &algebra).unwrap();

    let last_seen = env.clock.now();
    env.clock.advance(TimeDelta::minutes(2));

    let report = report_return(&env.state, &env.clock, last_seen);
    assert_eq!(report.away_seconds, 120);
    assert!(!report.needs_decision);
}

#[test]
fn a_long_absence_asks_the_student() {
    let env = env();
    let algebra = subject(&env, "Алгебра");
    start(&env.db, &env.state, &env.platform, &env.clock, &algebra).unwrap();

    let last_seen = env.clock.now();
    env.clock.advance(TimeDelta::hours(1));

    assert!(report_return(&env.state, &env.clock, last_seen).needs_decision);
}

#[test]
fn nothing_is_asked_when_no_session_is_running() {
    let env = env();
    let last_seen = env.clock.now();
    env.clock.advance(TimeDelta::hours(1));

    let report = report_return(&env.state, &env.clock, last_seen);

    assert_eq!(report.away_seconds, 3600);
    assert!(!report.needs_decision);
}

#[test]
fn discarded_away_time_is_stored_as_paused() {
    let env = env();
    let algebra = subject(&env, "Алгебра");
    start(&env.db, &env.state, &env.platform, &env.clock, &algebra).unwrap();
    env.clock.advance(TimeDelta::minutes(5));

    let last_seen = env.clock.now();
    env.clock.advance(TimeDelta::hours(1));

    let view = discard_away(&env.state, &env.clock, last_seen).unwrap();
    assert_eq!(view.elapsed_seconds, 5 * 60);

    stop(&env.db, &env.state, &env.platform, &env.clock).unwrap();
    let stored = rows(&env, "2026-08-21");
    assert_eq!(stored[0].active_seconds, 5 * 60);
    assert_eq!(stored[0].paused_seconds, 60 * 60);
}

#[test]
fn keeping_away_time_leaves_it_counted() {
    let env = env();
    let algebra = subject(&env, "Алгебра");
    start(&env.db, &env.state, &env.platform, &env.clock, &algebra).unwrap();
    env.clock.advance(TimeDelta::hours(1));

    // The student chose «засчитать»: nothing is called, and the hour stands.
    let view = current(&env.db, &env.state, &env.platform, &env.clock)
        .unwrap()
        .unwrap();

    assert_eq!(view.elapsed_seconds, 3600);
}

// --- время с начала сессии -------------------------------------------------

#[test]
fn the_session_total_matches_the_phase_while_the_first_phase_runs() {
    let env = env();
    let algebra = subject(&env, "Алгебра");
    start(&env.db, &env.state, &env.platform, &env.clock, &algebra).unwrap();
    env.clock.advance(TimeDelta::minutes(12));

    let view = current(&env.db, &env.state, &env.platform, &env.clock)
        .unwrap()
        .unwrap();

    assert_eq!(view.session_seconds, view.elapsed_seconds);
    assert_eq!(view.session_seconds, 12 * 60);
}

#[test]
fn the_session_total_carries_over_from_one_work_phase_to_the_next() {
    let env = env();
    let algebra = subject(&env, "Алгебра");
    preset(
        &env,
        PresetInput {
            auto_start_next: true,
            is_default: true,
            ..pomodoro("Автоматический")
        },
    );
    start(&env.db, &env.state, &env.platform, &env.clock, &algebra).unwrap();

    // Работа, перерыв, снова работа — на счётчике только работа.
    env.clock.advance(TimeDelta::minutes(25));
    current(&env.db, &env.state, &env.platform, &env.clock).unwrap();
    env.clock.advance(TimeDelta::minutes(5));
    current(&env.db, &env.state, &env.platform, &env.clock).unwrap();
    env.clock.advance(TimeDelta::minutes(10));

    let view = current(&env.db, &env.state, &env.platform, &env.clock)
        .unwrap()
        .unwrap();

    assert_eq!(view.phase, "work");
    assert_eq!(view.elapsed_seconds, 10 * 60);
    assert_eq!(view.session_seconds, 35 * 60);
}

#[test]
fn a_break_does_not_add_to_the_session_total() {
    let env = env();
    let algebra = subject(&env, "Алгебра");
    preset(
        &env,
        PresetInput {
            is_default: true,
            ..pomodoro("Классический")
        },
    );
    start(&env.db, &env.state, &env.platform, &env.clock, &algebra).unwrap();
    env.clock.advance(TimeDelta::minutes(25));
    skip_phase(&env.db, &env.state, &env.platform, &env.clock).unwrap();

    env.clock.advance(TimeDelta::minutes(4));
    let view = current(&env.db, &env.state, &env.platform, &env.clock)
        .unwrap()
        .unwrap();

    assert_eq!(view.phase, "break");
    assert_eq!(view.elapsed_seconds, 4 * 60);
    assert_eq!(view.session_seconds, 25 * 60);
}

#[test]
fn paused_time_is_left_out_of_the_session_total() {
    let env = env();
    let algebra = subject(&env, "Алгебра");
    start(&env.db, &env.state, &env.platform, &env.clock, &algebra).unwrap();
    env.clock.advance(TimeDelta::minutes(10));
    pause(&env.state, &env.platform, &env.clock).unwrap();
    env.clock.advance(TimeDelta::minutes(30));
    resume(&env.state, &env.platform, &env.clock).unwrap();
    env.clock.advance(TimeDelta::minutes(5));

    let view = current(&env.db, &env.state, &env.platform, &env.clock)
        .unwrap()
        .unwrap();

    assert_eq!(view.session_seconds, 15 * 60);
}

// --- граница учебного дня --------------------------------------------------

#[test]
fn a_late_session_belongs_to_the_previous_study_day_when_the_boundary_says_so() {
    let env = env();
    write_day(&env.db, 4 * 60 * 60).unwrap();
    let algebra = subject(&env, "Алгебра");
    // 01:30 — по календарю уже 22-е, по учебному дню ещё 21-е.
    let local_half_past_one = chrono::Local
        .with_ymd_and_hms(2026, 8, 22, 1, 30, 0)
        .single()
        .expect("01:30 22 августа однозначны в любом часовом поясе");
    env.clock.set(local_half_past_one.with_timezone(&Utc));

    start(&env.db, &env.state, &env.platform, &env.clock, &algebra).unwrap();
    env.clock.advance(TimeDelta::minutes(20));
    stop(&env.db, &env.state, &env.platform, &env.clock).unwrap();

    assert_eq!(rows(&env, "2026-08-21").len(), 1);
    assert!(rows(&env, "2026-08-22").is_empty());
}

#[test]
fn a_session_running_through_the_boundary_is_split_there_and_not_at_midnight() {
    let env = env();
    write_day(&env.db, 4 * 60 * 60).unwrap();
    let algebra = subject(&env, "Алгебра");
    let boundary = chrono::Local
        .with_ymd_and_hms(2026, 8, 22, 4, 0, 0)
        .single()
        .expect("04:00 22 августа однозначны в любом часовом поясе");
    env.clock
        .set(boundary.with_timezone(&Utc) - TimeDelta::minutes(30));

    start(&env.db, &env.state, &env.platform, &env.clock, &algebra).unwrap();
    env.clock.advance(TimeDelta::hours(1));
    stop(&env.db, &env.state, &env.platform, &env.clock).unwrap();

    let before = rows(&env, "2026-08-21");
    let after = rows(&env, "2026-08-22");
    assert_eq!(before.len(), 1);
    assert_eq!(after.len(), 1);
    assert_eq!(before[0].active_seconds, 30 * 60);
    assert_eq!(after[0].active_seconds, 30 * 60);
}

// --- незаписанное время текущей фазы ---------------------------------------

/// Учебный день часов при полуночной границе.
fn day_of(env: &Env) -> String {
    day_key(env.clock.now(), &chrono::Local, TimeDelta::zero())
}

#[test]
fn without_a_session_nothing_is_in_progress() {
    let env = env();

    assert_eq!(
        work_in_progress(&env.state, &env.clock, TimeDelta::zero(), "2026-08-21"),
        None
    );
}

#[test]
fn a_running_work_phase_reports_what_it_has_earned_so_far() {
    let env = env();
    let algebra = subject(&env, "Алгебра");
    start(&env.db, &env.state, &env.platform, &env.clock, &algebra).unwrap();
    env.clock.advance(TimeDelta::minutes(18));

    let day = day_of(&env);
    assert_eq!(
        work_in_progress(&env.state, &env.clock, TimeDelta::zero(), &day),
        Some((algebra, 18 * 60))
    );
}

#[test]
fn paused_time_is_not_earned() {
    let env = env();
    let algebra = subject(&env, "Алгебра");
    start(&env.db, &env.state, &env.platform, &env.clock, &algebra).unwrap();
    env.clock.advance(TimeDelta::minutes(10));
    pause(&env.state, &env.platform, &env.clock).unwrap();
    env.clock.advance(TimeDelta::minutes(45));

    let day = day_of(&env);
    assert_eq!(
        work_in_progress(&env.state, &env.clock, TimeDelta::zero(), &day),
        Some((algebra, 10 * 60))
    );
}

#[test]
fn a_break_earns_the_day_nothing() {
    let env = env();
    let algebra = subject(&env, "Алгебра");
    preset(
        &env,
        PresetInput {
            is_default: true,
            ..pomodoro("Классический")
        },
    );
    start(&env.db, &env.state, &env.platform, &env.clock, &algebra).unwrap();
    env.clock.advance(TimeDelta::minutes(25));
    skip_phase(&env.db, &env.state, &env.platform, &env.clock).unwrap();
    env.clock.advance(TimeDelta::minutes(4));

    let day = day_of(&env);
    assert_eq!(
        work_in_progress(&env.state, &env.clock, TimeDelta::zero(), &day),
        None
    );
}

#[test]
fn a_phase_that_crossed_the_boundary_earns_each_day_its_own_part() {
    let env = env();
    let algebra = subject(&env, "Алгебра");
    let local_midnight = chrono::Local
        .with_ymd_and_hms(2026, 8, 22, 0, 0, 0)
        .single()
        .expect("полночь 22 августа однозначна в любом часовом поясе");
    env.clock
        .set(local_midnight.with_timezone(&Utc) - TimeDelta::minutes(40));

    start(&env.db, &env.state, &env.platform, &env.clock, &algebra).unwrap();
    env.clock.advance(TimeDelta::hours(1));

    assert_eq!(
        work_in_progress(&env.state, &env.clock, TimeDelta::zero(), "2026-08-21"),
        Some((algebra.clone(), 40 * 60))
    );
    assert_eq!(
        work_in_progress(&env.state, &env.clock, TimeDelta::zero(), "2026-08-22"),
        Some((algebra, 20 * 60))
    );
}
