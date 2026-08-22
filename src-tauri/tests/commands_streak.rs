//! Tests for the streak command: what the page is given, and what the daily
//! minimum does to it.
//!
//! The walk itself is tested in `streak.rs` without a database. Here it is
//! the wiring: sessions read, settings applied, calendar built around the
//! study day the app thinks it is.

use chrono::{TimeDelta, TimeZone, Utc};
use lokked_lib::commands::settings::write_day;
use lokked_lib::commands::streak::{read_streak, save_image, streak_page, write_streak};
use lokked_lib::commands::ErrorKind;
use lokked_lib::core::stats::streak::{DayState, STREAK_MIN_SECONDS};
use lokked_lib::db::sessions::{NewSession, SessionRepo};
use lokked_lib::db::subjects::SubjectRepo;
use lokked_lib::db::Database;

/// The day every test treats as today.
const TODAY: &str = "2026-08-21";

fn new_db() -> Database {
    Database::open_in_memory().expect("in-memory database should open")
}

fn subject(db: &Database) -> String {
    SubjectRepo::new(db)
        .create("Матанализ", None, None, 0)
        .unwrap()
        .id
}

/// Writes one finished work phase against `day`.
fn study(db: &Database, subject_id: &str, day: &str, active_seconds: i64) {
    let started = Utc.with_ymd_and_hms(2026, 8, 21, 9, 0, 0).unwrap();

    SessionRepo::new(db)
        .create(NewSession {
            subject_id,
            preset_id: None,
            mode: "countup",
            phase: "work",
            started_at: started,
            ended_at: started + TimeDelta::seconds(active_seconds),
            day_key: day,
            active_seconds,
            paused_seconds: 0,
            planned_seconds: None,
            completed: false,
            interruptions: 0,
            device_id: None,
        })
        .unwrap();
}

/// Studies `count` days in a row, ending on `last`.
fn study_run(db: &Database, subject_id: &str, last: &str, count: i64) {
    let last = chrono::NaiveDate::parse_from_str(last, "%Y-%m-%d").unwrap();

    for back in 0..count {
        let day = (last - TimeDelta::days(back))
            .format("%Y-%m-%d")
            .to_string();
        study(db, subject_id, &day, 45 * 60);
    }
}

#[test]
fn a_database_with_nothing_in_it_shows_an_empty_page() {
    let db = new_db();

    let page = streak_page(&db, TODAY).unwrap();

    assert_eq!(page.current, 0);
    assert_eq!(page.longest, 0);
    assert_eq!(page.freezes, 0);
    assert_eq!(page.today_seconds, 0);
    assert_eq!(page.min_seconds, STREAK_MIN_SECONDS);
    // Календарь всё равно рисуется: пустой август, а не пустой экран.
    assert_eq!(page.month.days.len(), 31);
    assert_eq!(page.month.year, 2026);
    assert_eq!(page.month.month, 8);
}

#[test]
fn a_run_of_days_becomes_a_streak_with_its_calendar() {
    let db = new_db();
    let subject = subject(&db);
    study_run(&db, &subject, TODAY, 12);

    let page = streak_page(&db, TODAY).unwrap();

    assert_eq!(page.current, 12);
    assert_eq!(page.longest, 12);
    assert_eq!(page.freezes, 1, "за десять дней подряд одна заморозка");
    assert_eq!(page.today_seconds, 45 * 60);
    assert_eq!(page.month.days[20].state, DayState::Counted);
    assert_eq!(page.month.days[21].state, DayState::Future);
}

#[test]
fn the_milestones_come_with_the_page() {
    let db = new_db();
    let subject = subject(&db);
    study_run(&db, &subject, TODAY, 12);

    let page = streak_page(&db, TODAY).unwrap();

    assert_eq!(page.milestones.len(), 3);
    assert!(page.milestones[0].reached);
    assert_eq!(page.milestones[1].remaining, 18);
}

#[test]
fn several_sessions_of_one_day_are_summed_before_the_minimum() {
    let db = new_db();
    let subject = subject(&db);
    study(&db, &subject, TODAY, 4 * 60);
    study(&db, &subject, TODAY, 7 * 60);

    let page = streak_page(&db, TODAY).unwrap();

    assert_eq!(page.today_seconds, 11 * 60);
    assert_eq!(page.current, 1);
}

#[test]
fn raising_the_minimum_can_end_a_streak_that_was_scraping_by() {
    let db = new_db();
    let subject = subject(&db);
    for day in ["2026-08-19", "2026-08-20", TODAY] {
        study(&db, &subject, day, 12 * 60);
    }

    assert_eq!(streak_page(&db, TODAY).unwrap().current, 3);

    write_streak(&db, 30 * 60).unwrap();

    let page = streak_page(&db, TODAY).unwrap();
    assert_eq!(page.current, 0);
    assert_eq!(page.min_seconds, 30 * 60);
}

#[test]
fn the_minimum_survives_a_round_trip() {
    let db = new_db();

    write_streak(&db, 25 * 60).unwrap();

    assert_eq!(read_streak(&db).unwrap().min_seconds, 25 * 60);
}

#[test]
fn a_minimum_the_screen_should_never_send_is_refused() {
    let db = new_db();

    assert_eq!(
        write_streak(&db, 60).unwrap_err().kind,
        ErrorKind::Validation
    );
    assert_eq!(
        write_streak(&db, 20 * 60 + 30).unwrap_err().kind,
        ErrorKind::Validation
    );
    assert_eq!(read_streak(&db).unwrap().min_seconds, STREAK_MIN_SECONDS);
}

#[test]
fn the_page_reports_the_day_boundary_it_counted_by() {
    let db = new_db();
    write_day(&db, 4 * 60 * 60).unwrap();

    assert_eq!(streak_page(&db, TODAY).unwrap().day_start_seconds, 4 * 3600);
}

#[test]
fn the_record_carries_the_days_it_ran_between() {
    let db = new_db();
    let subject = subject(&db);
    study_run(&db, &subject, "2026-04-10", 27);
    study_run(&db, &subject, TODAY, 3);

    let page = streak_page(&db, TODAY).unwrap();

    assert_eq!(page.current, 3);
    assert_eq!(page.longest, 27);
    assert_eq!(page.longest_from.as_deref(), Some("2026-03-15"));
    assert_eq!(page.longest_to.as_deref(), Some("2026-04-10"));
}

// --- картинка «поделиться» -------------------------------------------------

#[test]
fn the_share_image_lands_in_the_directory_it_was_given() {
    let directory = std::env::temp_dir().join(format!("lokked-share-{}", std::process::id()));

    // Однопиксельный PNG, как его отдаёт canvas.toDataURL.
    let png = "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mP8z8BQDwAEhQGAhKmMIQAAAABJRU5ErkJggg==";
    let path = save_image(directory.clone(), png, TODAY).unwrap();

    assert!(path.ends_with("lokked-streak-2026-08-21.png"), "{path}");
    let written = std::fs::read(&path).unwrap();
    assert_eq!(&written[..4], b"\x89PNG", "на диск лёг именно PNG");

    std::fs::remove_dir_all(directory).ok();
}

#[test]
fn a_broken_image_is_refused_rather_than_written() {
    let directory = std::env::temp_dir().join(format!("lokked-broken-{}", std::process::id()));

    let error = save_image(directory.clone(), "data:image/png;base64,????", TODAY).unwrap_err();

    assert_eq!(error.kind, ErrorKind::Validation);
    assert!(
        !directory.exists(),
        "испорченная картинка не создаёт каталог"
    );
}

#[test]
fn an_empty_image_is_refused() {
    let directory = std::env::temp_dir().join(format!("lokked-empty-{}", std::process::id()));

    let error = save_image(directory, "data:image/png;base64,", TODAY).unwrap_err();

    assert_eq!(error.kind, ErrorKind::Validation);
}
