//! Tests for the queries the statistics screen reads: time over a period,
//! answers per day, per-card accuracy, and where «всё время» begins.

use chrono::{TimeDelta, TimeZone, Utc};
use lokked_lib::db::cards::{CardRepo, NewCard};
use lokked_lib::db::decks::DeckRepo;
use lokked_lib::db::reviews::{NewReview, ReviewRepo};
use lokked_lib::db::sessions::{NewSession, SessionRepo};
use lokked_lib::db::subjects::SubjectRepo;
use lokked_lib::db::Database;

fn new_db() -> Database {
    Database::open_in_memory().expect("in-memory database should open")
}

fn subject(db: &Database, name: &str) -> String {
    SubjectRepo::new(db).create(name, None, None, 0).unwrap().id
}

fn card(db: &Database, front: &str) -> String {
    let deck = DeckRepo::new(db).create(None, "Колода", None).unwrap();

    CardRepo::new(db)
        .create(NewCard {
            deck_id: &deck.id,
            front,
            back: "ответ",
            hint: None,
            tags: None,
        })
        .unwrap()
        .id
}

/// Records one work session of `active_seconds` on `day`.
fn session(db: &Database, subject_id: &str, day: &str, active_seconds: i64) {
    session_as(
        db,
        subject_id,
        day,
        active_seconds,
        "work",
        "countup",
        false,
    );
}

fn session_as(
    db: &Database,
    subject_id: &str,
    day: &str,
    active_seconds: i64,
    phase: &str,
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

fn review(db: &Database, card_id: &str, day: &str, correct: bool) {
    ReviewRepo::new(db)
        .create(NewReview {
            card_id,
            reviewed_at: Utc.with_ymd_and_hms(2026, 8, 21, 9, 0, 0).unwrap(),
            day_key: day,
            result: if correct { "good" } else { "again" },
            correct,
            mode: "classic",
            think_ms: Some(1500),
            total_ms: Some(2500),
            device_id: None,
        })
        .unwrap();
}

// --- SessionRepo -----------------------------------------------------------

#[test]
fn time_per_subject_is_summed_over_the_whole_period() {
    let db = new_db();
    let maths = subject(&db, "Математика");
    session(&db, &maths, "2026-08-19", 600);
    session(&db, &maths, "2026-08-21", 1200);

    let totals = SessionRepo::new(&db)
        .active_seconds_by_subject_range("2026-08-19", "2026-08-21")
        .unwrap();

    assert_eq!(totals, vec![(maths, 1800)]);
}

#[test]
fn time_outside_the_period_is_not_counted() {
    let db = new_db();
    let maths = subject(&db, "Математика");
    session(&db, &maths, "2026-08-01", 3600);
    session(&db, &maths, "2026-08-20", 600);

    let totals = SessionRepo::new(&db)
        .active_seconds_by_subject_range("2026-08-19", "2026-08-21")
        .unwrap();

    assert_eq!(totals, vec![(maths, 600)]);
}

#[test]
fn breaks_do_not_count_as_study_time_over_a_period() {
    let db = new_db();
    let maths = subject(&db, "Математика");
    session_as(&db, &maths, "2026-08-20", 900, "break", "pomodoro", true);

    let totals = SessionRepo::new(&db)
        .active_seconds_by_subject_range("2026-08-19", "2026-08-21")
        .unwrap();

    assert!(totals.is_empty());
}

#[test]
fn only_finished_pomodoro_work_phases_are_counted_over_a_period() {
    let db = new_db();
    let maths = subject(&db, "Математика");
    session_as(&db, &maths, "2026-08-19", 1500, "work", "pomodoro", true);
    session_as(&db, &maths, "2026-08-20", 1500, "work", "pomodoro", true);
    // Брошенный на середине — не помидор.
    session_as(&db, &maths, "2026-08-20", 400, "work", "pomodoro", false);
    // Секундомер тоже не помидор, даже доведённый до конца.
    session_as(&db, &maths, "2026-08-20", 1500, "work", "countup", true);
    // И то, что было до периода.
    session_as(&db, &maths, "2026-08-01", 1500, "work", "pomodoro", true);

    let counted = SessionRepo::new(&db)
        .completed_pomodoros_range("2026-08-19", "2026-08-21")
        .unwrap();

    assert_eq!(counted, 2);
}

#[test]
fn the_earliest_session_day_is_where_all_time_begins() {
    let db = new_db();
    let maths = subject(&db, "Математика");
    session(&db, &maths, "2026-08-20", 600);
    session(&db, &maths, "2026-03-01", 600);

    assert_eq!(
        SessionRepo::new(&db).earliest_day().unwrap(),
        Some("2026-03-01".to_string())
    );
}

#[test]
fn a_database_without_sessions_has_no_earliest_day() {
    assert_eq!(SessionRepo::new(&new_db()).earliest_day().unwrap(), None);
}

// --- ReviewRepo ------------------------------------------------------------

#[test]
fn answers_are_counted_per_day_with_the_correct_ones_apart() {
    let db = new_db();
    let card_id = card(&db, "front");
    review(&db, &card_id, "2026-08-20", true);
    review(&db, &card_id, "2026-08-20", false);
    review(&db, &card_id, "2026-08-21", true);

    let counted = ReviewRepo::new(&db)
        .counts_by_day("2026-08-19", "2026-08-21")
        .unwrap();

    assert_eq!(
        counted,
        vec![
            ("2026-08-20".to_string(), 2, 1),
            ("2026-08-21".to_string(), 1, 1),
        ]
    );
}

#[test]
fn a_day_nobody_answered_anything_is_absent_rather_than_zero() {
    let db = new_db();
    let card_id = card(&db, "front");
    review(&db, &card_id, "2026-08-21", true);

    let counted = ReviewRepo::new(&db)
        .counts_by_day("2026-08-19", "2026-08-21")
        .unwrap();

    assert_eq!(counted.len(), 1);
}

#[test]
fn per_card_accuracy_covers_every_deck_in_the_period() {
    let db = new_db();
    let first = card(&db, "первая");
    let second = card(&db, "вторая");
    review(&db, &first, "2026-08-20", true);
    review(&db, &first, "2026-08-20", false);
    review(&db, &second, "2026-08-21", true);

    let mut stats = ReviewRepo::new(&db)
        .accuracy_by_card_in_days("2026-08-19", "2026-08-21")
        .unwrap();
    stats.sort();

    let mut expected = vec![(first, 2, 1), (second, 1, 1)];
    expected.sort();
    assert_eq!(stats, expected);
}

#[test]
fn a_deleted_card_drops_out_of_the_accuracy_list() {
    let db = new_db();
    let card_id = card(&db, "front");
    review(&db, &card_id, "2026-08-20", false);
    CardRepo::new(&db).soft_delete(&card_id).unwrap();

    let stats = ReviewRepo::new(&db)
        .accuracy_by_card_in_days("2026-08-19", "2026-08-21")
        .unwrap();

    assert!(stats.is_empty());
}

#[test]
fn the_earliest_answer_day_is_where_all_time_begins() {
    let db = new_db();
    let card_id = card(&db, "front");
    review(&db, &card_id, "2026-08-21", true);
    review(&db, &card_id, "2026-05-05", true);

    assert_eq!(
        ReviewRepo::new(&db).earliest_day().unwrap(),
        Some("2026-05-05".to_string())
    );
}

// --- CardRepo --------------------------------------------------------------

#[test]
fn cards_are_looked_up_by_a_list_of_ids_in_one_go() {
    let db = new_db();
    let first = card(&db, "первая");
    let second = card(&db, "вторая");
    card(&db, "третья");

    let found = CardRepo::new(&db)
        .list_by_ids(&[first.clone(), second.clone()])
        .unwrap();

    let mut ids: Vec<String> = found.into_iter().map(|card| card.id).collect();
    ids.sort();
    let mut expected = vec![first, second];
    expected.sort();
    assert_eq!(ids, expected);
}

#[test]
fn looking_up_no_ids_asks_the_database_nothing() {
    assert!(CardRepo::new(&new_db())
        .list_by_ids(&[])
        .unwrap()
        .is_empty());
}
