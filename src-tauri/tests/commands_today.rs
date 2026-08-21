//! Tests for the «сегодня» totals: what counts toward today's study time,
//! and where the day boundary falls.

use chrono::{TimeDelta, TimeZone, Utc};
use lokked_lib::commands::today::totals;
use lokked_lib::core::clock::FakeClock;
use lokked_lib::core::dayline::day_key;
use lokked_lib::db::sessions::{NewSession, SessionRepo};
use lokked_lib::db::subjects::SubjectRepo;
use lokked_lib::db::Database;

fn new_db() -> Database {
    Database::open_in_memory().expect("in-memory database should open")
}

/// Records one session on `day`, with the given phase and active time.
fn record(db: &Database, subject_id: &str, day: &str, phase: &str, active_seconds: i64) {
    let started = Utc.with_ymd_and_hms(2026, 8, 21, 9, 0, 0).unwrap();

    SessionRepo::new(db)
        .create(NewSession {
            subject_id,
            preset_id: None,
            mode: "countup",
            phase,
            started_at: started,
            ended_at: started + TimeDelta::seconds(active_seconds),
            day_key: day,
            active_seconds,
            paused_seconds: 0,
            planned_seconds: None,
            completed: true,
            interruptions: 0,
            device_id: None,
        })
        .unwrap();
}

/// The clock's study day under the given `day_start`, in the machine's own
/// timezone — the same conversion the command does, so these tests pass
/// wherever they run.
fn today(clock: &FakeClock, day_start: TimeDelta) -> String {
    use chrono::Local;
    use lokked_lib::core::clock::Clock;

    day_key(clock.now(), &Local, day_start)
}

#[test]
fn a_day_with_nothing_recorded_reports_no_subjects() {
    let db = new_db();
    let clock = FakeClock::new(Utc.with_ymd_and_hms(2026, 8, 21, 12, 0, 0).unwrap());

    let result = totals(&db, &clock, TimeDelta::zero()).unwrap();

    assert_eq!(result.day_key, today(&clock, TimeDelta::zero()));
    assert!(result.seconds_by_subject.is_empty());
}

#[test]
fn sessions_for_one_subject_are_summed() {
    let db = new_db();
    let clock = FakeClock::new(Utc.with_ymd_and_hms(2026, 8, 21, 12, 0, 0).unwrap());
    let subject = SubjectRepo::new(&db)
        .create("Алгебра", Some("subject-1"), None, 0)
        .unwrap();
    let day = today(&clock, TimeDelta::zero());

    record(&db, &subject.id, &day, "work", 25 * 60);
    record(&db, &subject.id, &day, "work", 15 * 60);

    let result = totals(&db, &clock, TimeDelta::zero()).unwrap();

    assert_eq!(result.seconds_by_subject, vec![(subject.id, 40 * 60)]);
}

#[test]
fn breaks_do_not_count_as_study_time() {
    let db = new_db();
    let clock = FakeClock::new(Utc.with_ymd_and_hms(2026, 8, 21, 12, 0, 0).unwrap());
    let subject = SubjectRepo::new(&db)
        .create("Алгебра", Some("subject-1"), None, 0)
        .unwrap();
    let day = today(&clock, TimeDelta::zero());

    record(&db, &subject.id, &day, "work", 25 * 60);
    record(&db, &subject.id, &day, "break", 5 * 60);
    record(&db, &subject.id, &day, "long_break", 15 * 60);

    let result = totals(&db, &clock, TimeDelta::zero()).unwrap();

    assert_eq!(result.seconds_by_subject, vec![(subject.id, 25 * 60)]);
}

#[test]
fn another_days_sessions_are_left_out() {
    let db = new_db();
    let clock = FakeClock::new(Utc.with_ymd_and_hms(2026, 8, 21, 12, 0, 0).unwrap());
    let subject = SubjectRepo::new(&db)
        .create("Алгебра", Some("subject-1"), None, 0)
        .unwrap();

    record(
        &db,
        &subject.id,
        &today(&clock, TimeDelta::zero()),
        "work",
        25 * 60,
    );
    record(&db, &subject.id, "2026-08-20", "work", 90 * 60);

    let result = totals(&db, &clock, TimeDelta::zero()).unwrap();

    assert_eq!(result.seconds_by_subject, vec![(subject.id, 25 * 60)]);
}

#[test]
fn every_subject_with_time_today_is_reported() {
    let db = new_db();
    let clock = FakeClock::new(Utc.with_ymd_and_hms(2026, 8, 21, 12, 0, 0).unwrap());
    let repo = SubjectRepo::new(&db);
    let algebra = repo.create("Алгебра", Some("subject-1"), None, 0).unwrap();
    let physics = repo.create("Физика", Some("subject-2"), None, 1).unwrap();
    let idle = repo.create("Химия", Some("subject-3"), None, 2).unwrap();
    let day = today(&clock, TimeDelta::zero());

    record(&db, &algebra.id, &day, "work", 30 * 60);
    record(&db, &physics.id, &day, "work", 10 * 60);

    let result = totals(&db, &clock, TimeDelta::zero()).unwrap();

    let mut seconds = result.seconds_by_subject;
    seconds.sort();
    let mut expected = vec![(algebra.id, 30 * 60), (physics.id, 10 * 60)];
    expected.sort();
    assert_eq!(seconds, expected);
    // A subject studied on no day at all is simply absent.
    assert!(!seconds.iter().any(|(id, _)| id == &idle.id));
}

#[test]
fn a_later_day_start_moves_the_reported_day_back() {
    use chrono::Local;

    let db = new_db();
    // 01:00 local, whatever local is on this machine: with the study day
    // starting at 04:00 that hour still belongs to the previous day, while
    // under a midnight boundary it does not.
    let local_one_am = Local
        .with_ymd_and_hms(2026, 8, 22, 1, 0, 0)
        .single()
        .expect("01:00 on 22 August is unambiguous in every timezone");
    let clock = FakeClock::new(local_one_am.with_timezone(&Utc));

    let midnight_boundary = totals(&db, &clock, TimeDelta::zero()).unwrap();
    let four_am_boundary = totals(&db, &clock, TimeDelta::hours(4)).unwrap();

    assert_eq!(midnight_boundary.day_key, "2026-08-22");
    assert_eq!(four_am_boundary.day_key, "2026-08-21");
}
