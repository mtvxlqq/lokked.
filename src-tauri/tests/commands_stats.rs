//! Tests for the statistics commands: what each tab adds up to, over which
//! period, and what the CSV export contains.

use chrono::{TimeDelta, TimeZone, Utc};
use lokked_lib::commands::stats::cards::{card_report, cards_stats, PROBLEM_LIMIT};
use lokked_lib::commands::stats::export::export_csv;
use lokked_lib::commands::stats::period;
use lokked_lib::commands::stats::time::time_stats;
use lokked_lib::core::review::Grade;
use lokked_lib::core::stats::time::StatsRange;
use lokked_lib::db::cards::{CardRepo, NewCard};
use lokked_lib::db::decks::DeckRepo;
use lokked_lib::db::reviews::{NewReview, ReviewRepo};
use lokked_lib::db::sessions::{NewSession, SessionRepo};
use lokked_lib::db::subjects::SubjectRepo;
use lokked_lib::db::Database;

/// The day every test treats as today.
const TODAY: &str = "2026-08-21";

fn new_db() -> Database {
    Database::open_in_memory().expect("in-memory database should open")
}

fn subject(db: &Database, name: &str) -> String {
    SubjectRepo::new(db).create(name, None, None, 0).unwrap().id
}

fn deck(db: &Database) -> String {
    DeckRepo::new(db).create(None, "Колода", None).unwrap().id
}

fn card(db: &Database, deck_id: &str, front: &str) -> String {
    CardRepo::new(db)
        .create(NewCard {
            deck_id,
            front,
            back: "ответ",
            hint: None,
            tags: None,
        })
        .unwrap()
        .id
}

fn study(db: &Database, subject_id: &str, day: &str, active_seconds: i64) {
    study_as(db, subject_id, day, active_seconds, "countup", false);
}

fn study_as(
    db: &Database,
    subject_id: &str,
    day: &str,
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
            phase: "work",
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

fn answer(db: &Database, card_id: &str, day: &str, grade: Grade) {
    answer_timed(db, card_id, day, grade, Some(1500));
}

fn answer_timed(db: &Database, card_id: &str, day: &str, grade: Grade, think_ms: Option<i64>) {
    ReviewRepo::new(db)
        .create(NewReview {
            card_id,
            reviewed_at: Utc.with_ymd_and_hms(2026, 8, 21, 9, 0, 0).unwrap(),
            day_key: day,
            result: grade.as_str(),
            correct: grade.is_correct(),
            mode: "classic",
            think_ms,
            total_ms: think_ms.map(|ms| ms + 1000),
            device_id: None,
        })
        .unwrap();
}

// --- период ----------------------------------------------------------------

#[test]
fn a_week_ends_today_and_starts_six_days_earlier() {
    let db = new_db();

    let period = period(&db, StatsRange::Week, TODAY).unwrap();

    assert_eq!(period.from, "2026-08-15");
    assert_eq!(period.to, TODAY);
}

#[test]
fn all_time_starts_at_the_first_record_of_either_kind() {
    let db = new_db();
    let maths = subject(&db, "Математика");
    let deck_id = deck(&db);
    let card_id = card(&db, &deck_id, "front");
    study(&db, &maths, "2026-06-01", 600);
    answer(&db, &card_id, "2026-04-01", Grade::Good);

    let period = period(&db, StatsRange::All, TODAY).unwrap();

    assert_eq!(period.from, "2026-04-01");
}

#[test]
fn all_time_in_an_empty_database_is_just_today() {
    let db = new_db();

    assert_eq!(period(&db, StatsRange::All, TODAY).unwrap().from, TODAY);
}

// --- вкладка «Время» -------------------------------------------------------

#[test]
fn the_time_tab_of_an_empty_database_is_all_zeroes() {
    let db = new_db();

    let stats = time_stats(&db, StatsRange::Month, TODAY).unwrap();

    assert_eq!(stats.total_seconds, 0);
    assert_eq!(stats.pomodoros, 0);
    assert_eq!(stats.streak_days, 0);
    assert!(stats.subjects.is_empty());
    // Тепловая карта всё равно есть: пустая сетка честнее, чем её отсутствие.
    // Двадцать девять полных недель плюс неделя, идущая по сегодняшний день.
    assert_eq!(stats.heatmap.len(), 29 * 7 + 5);
}

#[test]
fn subjects_are_summed_over_the_period_and_ordered_by_time() {
    let db = new_db();
    let maths = subject(&db, "Математика");
    let physics = subject(&db, "Физика");
    study(&db, &maths, "2026-08-19", 600);
    study(&db, &maths, TODAY, 1200);
    study(&db, &physics, TODAY, 900);

    let stats = time_stats(&db, StatsRange::Week, TODAY).unwrap();

    assert_eq!(stats.total_seconds, 2700);
    assert_eq!(stats.subjects.len(), 2);
    assert_eq!(stats.subjects[0].subject_id, maths);
    assert_eq!(stats.subjects[0].seconds, 1800);
    assert_eq!(stats.subjects[1].subject_id, physics);
}

#[test]
fn a_narrower_period_leaves_the_earlier_days_out() {
    let db = new_db();
    let maths = subject(&db, "Математика");
    study(&db, &maths, "2026-08-19", 600);
    study(&db, &maths, TODAY, 1200);

    let stats = time_stats(&db, StatsRange::Day, TODAY).unwrap();

    assert_eq!(stats.total_seconds, 1200);
}

#[test]
fn finished_pomodoros_are_counted_over_the_period() {
    let db = new_db();
    let maths = subject(&db, "Математика");
    study_as(&db, &maths, TODAY, 1500, "pomodoro", true);
    study_as(&db, &maths, TODAY, 1500, "pomodoro", false);

    assert_eq!(
        time_stats(&db, StatsRange::Day, TODAY).unwrap().pomodoros,
        1
    );
}

#[test]
fn the_streak_is_longer_than_the_period_it_is_shown_next_to() {
    let db = new_db();
    let maths = subject(&db, "Математика");
    // Десять дней подряд по часу, кончая сегодняшним.
    for day in 12..=21 {
        study(&db, &maths, &format!("2026-08-{day:02}"), 3600);
    }

    let stats = time_stats(&db, StatsRange::Day, TODAY).unwrap();

    assert_eq!(stats.streak_days, 10);
    // Период — один день, серия — десять: одно другому не подчинено.
    assert_eq!(stats.total_seconds, 3600);
}

#[test]
fn the_heatmap_ends_today_and_starts_on_a_monday() {
    let db = new_db();
    let maths = subject(&db, "Математика");
    study(&db, &maths, TODAY, 3600);

    let stats = time_stats(&db, StatsRange::Day, TODAY).unwrap();

    assert_eq!(stats.heatmap.first().unwrap().day_key, "2026-01-26");
    assert_eq!(stats.heatmap.first().unwrap().weekday, 0);

    // Карта кончается сегодняшним днём, а не концом недели: рисовать клетки
    // на ещё не наступившие дни незачем.
    let last = stats.heatmap.last().unwrap();
    assert_eq!(last.day_key, TODAY);
    assert_eq!(last.seconds, 3600);
    assert!(last.level > 0);
}

// --- вкладка «Карточки» ----------------------------------------------------

#[test]
fn the_cards_tab_of_an_empty_database_shows_no_accuracy() {
    let db = new_db();

    let stats = cards_stats(&db, StatsRange::Week, TODAY).unwrap();

    assert_eq!(stats.answered, 0);
    assert_eq!(stats.correct, 0);
    assert_eq!(stats.accuracy_percent, 0);
    assert!(stats.problems.is_empty());
    // По точке на каждый день недели, даже пустой.
    assert_eq!(stats.by_day.len(), 7);
}

#[test]
fn answers_over_the_period_add_up_to_an_accuracy() {
    let db = new_db();
    let deck_id = deck(&db);
    let card_id = card(&db, &deck_id, "front");
    answer(&db, &card_id, "2026-08-20", Grade::Good);
    answer(&db, &card_id, "2026-08-20", Grade::Again);
    answer(&db, &card_id, TODAY, Grade::Hard);

    let stats = cards_stats(&db, StatsRange::Week, TODAY).unwrap();

    assert_eq!(stats.answered, 3);
    // «С трудом» — это вспомнил.
    assert_eq!(stats.correct, 2);
    assert_eq!(stats.accuracy_percent, 67);

    let today = stats.by_day.last().unwrap();
    assert_eq!(today.day_key, TODAY);
    assert_eq!(today.answered, 1);
    assert_eq!(today.accuracy_percent, 100);
}

#[test]
fn the_problem_list_carries_the_front_of_every_card_it_names() {
    let db = new_db();
    let deck_id = deck(&db);
    let weak = card(&db, &deck_id, "Критерий обратимости");
    let known = card(&db, &deck_id, "Дважды два");
    for _ in 0..3 {
        answer(&db, &weak, TODAY, Grade::Again);
        answer(&db, &known, TODAY, Grade::Good);
    }

    let stats = cards_stats(&db, StatsRange::Day, TODAY).unwrap();

    assert_eq!(stats.problems.len(), 2);
    assert_eq!(stats.problems[0].card.card_id, weak);
    assert_eq!(stats.problems[0].front, "Критерий обратимости");
    assert_eq!(stats.problems[0].deck_id, deck_id);
    assert_eq!(stats.problems[0].card.accuracy_percent, 0);
    assert_eq!(stats.problems[1].card.card_id, known);
}

#[test]
fn a_card_seen_once_is_not_yet_a_problem() {
    let db = new_db();
    let deck_id = deck(&db);
    let fresh = card(&db, &deck_id, "Впервые вижу");
    answer(&db, &fresh, TODAY, Grade::Again);

    assert!(cards_stats(&db, StatsRange::Day, TODAY)
        .unwrap()
        .problems
        .is_empty());
}

#[test]
fn the_problem_list_stops_at_twenty_cards() {
    let db = new_db();
    let deck_id = deck(&db);
    for index in 0..PROBLEM_LIMIT + 5 {
        let card_id = card(&db, &deck_id, &format!("карточка {index}"));
        for _ in 0..3 {
            answer(&db, &card_id, TODAY, Grade::Again);
        }
    }

    let stats = cards_stats(&db, StatsRange::Day, TODAY).unwrap();

    assert_eq!(stats.problems.len(), PROBLEM_LIMIT);
}

// --- вкладка «Карточка» ----------------------------------------------------

#[test]
fn one_cards_history_reports_its_accuracy_streak_and_recall_time() {
    let db = new_db();
    let deck_id = deck(&db);
    let card_id = card(&db, &deck_id, "Критерий обратимости");
    answer_timed(&db, &card_id, "2026-08-19", Grade::Again, Some(4000));
    answer_timed(&db, &card_id, "2026-08-20", Grade::Good, Some(2000));
    answer_timed(&db, &card_id, TODAY, Grade::Easy, None);

    let report = card_report(&db, &card_id).unwrap();

    assert_eq!(report.front, "Критерий обратимости");
    assert_eq!(report.back, "ответ");
    assert_eq!(report.deck_id, deck_id);
    assert_eq!(report.stats.shown, 3);
    assert_eq!(report.stats.correct, 2);
    assert_eq!(report.stats.accuracy_percent, 67);
    assert_eq!(report.stats.current_streak, 2);
    assert_eq!(report.stats.average_think_ms, Some(3000));
    assert_eq!(
        report.stats.recent,
        vec![Grade::Again, Grade::Good, Grade::Easy]
    );
}

#[test]
fn a_card_nobody_has_answered_still_has_a_report() {
    let db = new_db();
    let deck_id = deck(&db);
    let card_id = card(&db, &deck_id, "Нетронутая");

    let report = card_report(&db, &card_id).unwrap();

    assert_eq!(report.stats.shown, 0);
    assert_eq!(report.stats.average_think_ms, None);
    assert!(report.stats.recent.is_empty());
}

#[test]
fn a_deleted_card_has_no_report() {
    let db = new_db();
    let deck_id = deck(&db);
    let card_id = card(&db, &deck_id, "Удалённая");
    CardRepo::new(&db).soft_delete(&card_id).unwrap();

    assert!(card_report(&db, &card_id).is_err());
    assert!(card_report(&db, "нет такой").is_err());
}

// --- экспорт ---------------------------------------------------------------

#[test]
fn the_export_has_a_line_per_day_of_the_period() {
    let db = new_db();
    let maths = subject(&db, "Математика");
    let deck_id = deck(&db);
    let card_id = card(&db, &deck_id, "front");
    study(&db, &maths, TODAY, 5400);
    answer(&db, &card_id, TODAY, Grade::Good);
    answer(&db, &card_id, TODAY, Grade::Again);

    let csv = export_csv(&db, StatsRange::Week, TODAY).unwrap();
    let lines: Vec<&str> = csv.lines().collect();

    // Заголовок и семь дней недели.
    assert_eq!(lines.len(), 8);
    assert!(lines[0].starts_with("день,"));
    assert_eq!(lines[1], "2026-08-15,0,0,0,0,");
    assert_eq!(lines[7], "2026-08-21,5400,90,2,1,50");
}
