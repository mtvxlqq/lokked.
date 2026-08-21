//! Tests for the session actions the desktop triggers: the `--toggle`
//! hotkey and the machine going to sleep mid-session.

use chrono::{TimeDelta, TimeZone, Utc};
use lokked_lib::commands::session::actions::{current, start, stop};
use lokked_lib::commands::session::desktop::{pause_for_sleep, toggle};
use lokked_lib::commands::session::SessionState;
use lokked_lib::commands::subjects::{self, SubjectInput};
use lokked_lib::core::clock::FakeClock;
use lokked_lib::db::sessions::SessionRepo;
use lokked_lib::db::Database;
use lokked_lib::platform::noop::NoopPlatform;
use lokked_lib::platform::SharedPlatform;

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

/// Starts a stopwatch session on a fresh subject.
fn running_session(env: &Env) -> String {
    let subject = subjects::create(
        &env.db,
        SubjectInput {
            name: "Математический анализ".to_string(),
            color: None,
            icon: None,
        },
    )
    .unwrap()
    .id;

    start(&env.db, &env.state, &env.platform, &env.clock, &subject).unwrap();

    subject
}

fn status(env: &Env) -> Option<String> {
    current(&env.db, &env.state, &env.platform, &env.clock)
        .unwrap()
        .map(|view| view.status)
}

// --- --toggle --------------------------------------------------------------

#[test]
fn toggling_a_running_session_pauses_it() {
    let env = env();
    running_session(&env);

    let view = toggle(&env.state, &env.platform, &env.clock).unwrap();

    assert_eq!(view.map(|view| view.status), Some("paused".to_string()));
    assert_eq!(status(&env), Some("paused".to_string()));
}

#[test]
fn toggling_a_paused_session_starts_it_again() {
    let env = env();
    running_session(&env);
    toggle(&env.state, &env.platform, &env.clock).unwrap();

    let view = toggle(&env.state, &env.platform, &env.clock).unwrap();

    assert_eq!(view.map(|view| view.status), Some("running".to_string()));
}

#[test]
fn a_pause_does_not_eat_the_time_that_was_already_studied() {
    let env = env();
    running_session(&env);
    env.clock.advance(TimeDelta::minutes(10));

    toggle(&env.state, &env.platform, &env.clock).unwrap();
    env.clock.advance(TimeDelta::minutes(30));
    let view = toggle(&env.state, &env.platform, &env.clock)
        .unwrap()
        .unwrap();

    // Полчаса на паузе не считаются, десять минут до неё — считаются.
    assert_eq!(view.session_seconds, 10 * 60);
}

#[test]
fn toggling_without_a_session_does_nothing_and_is_not_an_error() {
    let env = env();

    assert_eq!(toggle(&env.state, &env.platform, &env.clock).unwrap(), None);
}

// --- сон машины ------------------------------------------------------------

#[test]
fn a_running_session_is_paused_when_the_machine_suspends() {
    let env = env();
    running_session(&env);

    assert!(pause_for_sleep(&env.state, &env.platform, &env.clock));
    assert_eq!(status(&env), Some("paused".to_string()));
}

#[test]
fn the_time_the_machine_slept_is_not_study_time() {
    let env = env();
    let subject = running_session(&env);
    env.clock.advance(TimeDelta::minutes(5));

    pause_for_sleep(&env.state, &env.platform, &env.clock);
    // Ноутбук закрыли на всю ночь.
    env.clock.advance(TimeDelta::hours(9));
    stop(&env.db, &env.state, &env.platform, &env.clock).unwrap();

    // Строк может быть две: сон перенёс конец фазы через границу учебного
    // дня, и она записалась по куску на день. Считается их сумма.
    let written = SessionRepo::new(&env.db)
        .list_for_subject(&subject)
        .unwrap();
    let studied: i64 = written.iter().map(|row| row.active_seconds).sum();
    assert_eq!(studied, 5 * 60);
}

#[test]
fn suspending_an_already_paused_session_changes_nothing() {
    let env = env();
    running_session(&env);
    toggle(&env.state, &env.platform, &env.clock).unwrap();

    // Пауза уже стоит — засыпание не должно ни ставить вторую, ни спрашивать
    // потом, продолжать ли: студент сам поставил её раньше.
    assert!(!pause_for_sleep(&env.state, &env.platform, &env.clock));
    assert_eq!(status(&env), Some("paused".to_string()));
}

#[test]
fn suspending_without_a_session_is_silent() {
    let env = env();

    assert!(!pause_for_sleep(&env.state, &env.platform, &env.clock));
}
