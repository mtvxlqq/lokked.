//! Tests for the «сегодня» summary: what counts toward today's study time,
//! where the day boundary falls, and what the numbers above the subject list
//! add up to.

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
    record_as(db, subject_id, day, phase, active_seconds, "countup", true);
}

/// The same, spelling out the mode and whether the phase was carried to its
/// end — that pair is what makes a row count as a finished Pomodoro.
fn record_as(
    db: &Database,
    subject_id: &str,
    day: &str,
    phase: &str,
    active_seconds: i64,
    mode: &str,
    completed: bool,
) {
    let started = Utc.with_ymd_and_hms(2026, 8, 21, 9, 0, 0).unwrap();

    SessionRepo::new(db)
        .create(NewSession {
            subject_id,
            preset_id: None,
            mode,
            phase,
            started_at: started,
            ended_at: started + TimeDelta::seconds(active_seconds),
            day_key: day,
            active_seconds,
            paused_seconds: 0,
            planned_seconds: None,
            completed,
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

    day_key(clock_now(clock), &Local, day_start)
}

/// `clock.now()` без импорта трейта в каждом тесте.
fn clock_now(clock: &FakeClock) -> chrono::DateTime<Utc> {
    use lokked_lib::core::clock::Clock;

    clock.now()
}

#[test]
fn a_day_with_nothing_recorded_reports_no_subjects() {
    let db = new_db();
    let clock = FakeClock::new(Utc.with_ymd_and_hms(2026, 8, 21, 12, 0, 0).unwrap());

    let result = totals(&db, clock_now(&clock), TimeDelta::zero(), None).unwrap();

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

    let result = totals(&db, clock_now(&clock), TimeDelta::zero(), None).unwrap();

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

    let result = totals(&db, clock_now(&clock), TimeDelta::zero(), None).unwrap();

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

    let result = totals(&db, clock_now(&clock), TimeDelta::zero(), None).unwrap();

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

    let result = totals(&db, clock_now(&clock), TimeDelta::zero(), None).unwrap();

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

    let midnight_boundary = totals(&db, clock_now(&clock), TimeDelta::zero(), None).unwrap();
    let four_am_boundary = totals(&db, clock_now(&clock), TimeDelta::hours(4), None).unwrap();

    assert_eq!(midnight_boundary.day_key, "2026-08-22");
    assert_eq!(four_am_boundary.day_key, "2026-08-21");
}

// --- числа над списком предметов -------------------------------------------

#[test]
fn the_total_is_everything_studied_today_across_subjects() {
    let db = new_db();
    let clock = FakeClock::new(Utc.with_ymd_and_hms(2026, 8, 21, 12, 0, 0).unwrap());
    let repo = SubjectRepo::new(&db);
    let algebra = repo.create("Алгебра", Some("subject-1"), None, 0).unwrap();
    let physics = repo.create("Физика", Some("subject-2"), None, 1).unwrap();
    let day = today(&clock, TimeDelta::zero());

    record(&db, &algebra.id, &day, "work", 30 * 60);
    record(&db, &physics.id, &day, "work", 10 * 60);
    record(&db, &algebra.id, &day, "break", 5 * 60);
    record(&db, &algebra.id, "2026-08-20", "work", 90 * 60);

    let result = totals(&db, clock_now(&clock), TimeDelta::zero(), None).unwrap();

    assert_eq!(result.total_seconds, 40 * 60);
}

#[test]
fn only_finished_pomodoro_work_phases_are_counted_as_pomodoros() {
    let db = new_db();
    let clock = FakeClock::new(Utc.with_ymd_and_hms(2026, 8, 21, 12, 0, 0).unwrap());
    let subject = SubjectRepo::new(&db)
        .create("Алгебра", Some("subject-1"), None, 0)
        .unwrap();
    let day = today(&clock, TimeDelta::zero());

    record_as(&db, &subject.id, &day, "work", 25 * 60, "pomodoro", true);
    record_as(&db, &subject.id, &day, "work", 25 * 60, "pomodoro", true);
    // Брошенный на середине помидор помидором не стал.
    record_as(&db, &subject.id, &day, "work", 8 * 60, "pomodoro", false);
    // Перерыв — тем более.
    record_as(&db, &subject.id, &day, "break", 5 * 60, "pomodoro", true);
    // Секундомер помидоры не выращивает, сколько бы ни шёл.
    record_as(&db, &subject.id, &day, "work", 60 * 60, "countup", true);

    let result = totals(&db, clock_now(&clock), TimeDelta::zero(), None).unwrap();

    assert_eq!(result.pomodoros, 2);
}

#[test]
fn the_streak_counts_days_in_a_row_with_enough_time() {
    let db = new_db();
    let clock = FakeClock::new(Utc.with_ymd_and_hms(2026, 8, 21, 12, 0, 0).unwrap());
    let subject = SubjectRepo::new(&db)
        .create("Алгебра", Some("subject-1"), None, 0)
        .unwrap();

    record(&db, &subject.id, "2026-08-19", "work", 20 * 60);
    record(&db, &subject.id, "2026-08-20", "work", 20 * 60);
    record(
        &db,
        &subject.id,
        &today(&clock, TimeDelta::zero()),
        "work",
        20 * 60,
    );

    let result = totals(&db, clock_now(&clock), TimeDelta::zero(), None).unwrap();

    assert_eq!(result.streak_days, 3);
}

#[test]
fn the_streak_does_not_reset_when_the_day_turns_over() {
    // Ровно то, ради чего «сегодня» — фильтр, а не обнуление: новый день
    // начался пустым, но серия остаётся вчерашней.
    let db = new_db();
    let clock = FakeClock::new(Utc.with_ymd_and_hms(2026, 8, 21, 0, 30, 0).unwrap());
    let subject = SubjectRepo::new(&db)
        .create("Алгебра", Some("subject-1"), None, 0)
        .unwrap();
    let yesterday = day_key(
        clock_now(&clock) - TimeDelta::days(1),
        &chrono::Local,
        TimeDelta::zero(),
    );
    let before = day_key(
        clock_now(&clock) - TimeDelta::days(2),
        &chrono::Local,
        TimeDelta::zero(),
    );

    record(&db, &subject.id, &before, "work", 30 * 60);
    record(&db, &subject.id, &yesterday, "work", 30 * 60);

    let result = totals(&db, clock_now(&clock), TimeDelta::zero(), None).unwrap();

    assert_eq!(result.total_seconds, 0);
    assert_eq!(result.streak_days, 2);
}

#[test]
fn the_next_boundary_is_where_the_setting_puts_it() {
    use chrono::Local;

    let db = new_db();
    let local_noon = Local
        .with_ymd_and_hms(2026, 8, 21, 12, 0, 0)
        .single()
        .expect("полдень 21 августа однозначен в любом часовом поясе");
    let clock = FakeClock::new(local_noon.with_timezone(&Utc));

    let midnight = totals(&db, clock_now(&clock), TimeDelta::zero(), None).unwrap();
    let four_am = totals(&db, clock_now(&clock), TimeDelta::hours(4), None).unwrap();

    assert_eq!(
        midnight.next_boundary.with_timezone(&Local),
        Local.with_ymd_and_hms(2026, 8, 22, 0, 0, 0).unwrap()
    );
    assert_eq!(
        four_am.next_boundary.with_timezone(&Local),
        Local.with_ymd_and_hms(2026, 8, 22, 4, 0, 0).unwrap()
    );
}

#[test]
fn the_phase_still_running_counts_toward_today() {
    // Строки появляются только когда фаза кончилась, а полчаса за плечами
    // студент уже видит на экране — значит, их надо показать и в списке.
    let db = new_db();
    let clock = FakeClock::new(Utc.with_ymd_and_hms(2026, 8, 21, 12, 0, 0).unwrap());
    let subject = SubjectRepo::new(&db)
        .create("Алгебра", Some("subject-1"), None, 0)
        .unwrap();
    let day = today(&clock, TimeDelta::zero());

    record(&db, &subject.id, &day, "work", 10 * 60);
    let running = Some((subject.id.clone(), 30 * 60));

    let result = totals(&db, clock_now(&clock), TimeDelta::zero(), running).unwrap();

    assert_eq!(result.seconds_by_subject, vec![(subject.id, 40 * 60)]);
    assert_eq!(result.total_seconds, 40 * 60);
}

#[test]
fn a_first_session_of_the_day_shows_up_before_it_is_written_down() {
    let db = new_db();
    let clock = FakeClock::new(Utc.with_ymd_and_hms(2026, 8, 21, 12, 0, 0).unwrap());
    let subject = SubjectRepo::new(&db)
        .create("Алгебра", Some("subject-1"), None, 0)
        .unwrap();

    let running = Some((subject.id.clone(), 12 * 60));
    let result = totals(&db, clock_now(&clock), TimeDelta::zero(), running).unwrap();

    assert_eq!(result.seconds_by_subject, vec![(subject.id, 12 * 60)]);
    // И серия начинается сегодня же, а не назавтра.
    assert_eq!(result.streak_days, 1);
}
