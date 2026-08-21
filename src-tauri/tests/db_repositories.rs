//! CRUD tests for `SubjectRepo`, `PresetRepo`, `SessionRepo`, `SettingsRepo`,
//! `DeckRepo`, `CardRepo` and `ReviewRepo`, plus the
//! cross-cutting behaviours the schema promises: soft delete hides rows from
//! `list` but not `get`, and foreign keys are actually enforced.
//!
//! Schema-application tests live in `tests/db_migrations.rs`.

use chrono::{TimeDelta, Utc};
use lokked_lib::db::cards::{CardRepo, NewCard};
use lokked_lib::db::decks::DeckRepo;
use lokked_lib::db::presets::{NewPreset, PresetRepo};
use lokked_lib::db::reviews::{NewReview, ReviewRepo};
use lokked_lib::db::sessions::{NewSession, SessionRepo};
use lokked_lib::db::settings::SettingsRepo;
use lokked_lib::db::subjects::SubjectRepo;
use lokked_lib::db::Database;

fn new_db() -> Database {
    Database::open_in_memory().expect("in-memory database should open")
}

// --- SubjectRepo -----------------------------------------------------------

#[test]
fn a_created_subject_is_returned_with_a_generated_id_and_matching_fields() {
    let db = new_db();
    let repo = SubjectRepo::new(&db);

    let subject = repo
        .create("Linear Algebra", Some("#4f46e5"), Some("sigma"), 2)
        .unwrap();

    assert!(!subject.id.is_empty());
    assert_eq!(subject.name, "Linear Algebra");
    assert_eq!(subject.color.as_deref(), Some("#4f46e5"));
    assert_eq!(subject.icon.as_deref(), Some("sigma"));
    assert_eq!(subject.position, 2);
    assert_eq!(subject.created_at, subject.updated_at);
    assert_eq!(subject.deleted_at, None);
}

#[test]
fn get_finds_a_subject_by_id_and_none_for_an_unknown_id() {
    let db = new_db();
    let repo = SubjectRepo::new(&db);
    let created = repo.create("Physics", None, None, 0).unwrap();

    assert_eq!(repo.get(&created.id).unwrap(), Some(created));
    assert_eq!(repo.get("does-not-exist").unwrap(), None);
}

#[test]
fn list_returns_subjects_ordered_by_position() {
    let db = new_db();
    let repo = SubjectRepo::new(&db);
    repo.create("Third", None, None, 2).unwrap();
    repo.create("First", None, None, 0).unwrap();
    repo.create("Second", None, None, 1).unwrap();

    let names: Vec<String> = repo.list().unwrap().into_iter().map(|s| s.name).collect();

    assert_eq!(names, vec!["First", "Second", "Third"]);
}

#[test]
fn update_changes_fields_and_bumps_updated_at_without_touching_created_at() {
    let db = new_db();
    let repo = SubjectRepo::new(&db);
    let created = repo.create("Chemistry", None, None, 0).unwrap();

    repo.update(&created.id, "Organic Chemistry", Some("#22c55e"), None, 5)
        .unwrap();
    let updated = repo.get(&created.id).unwrap().unwrap();

    assert_eq!(updated.name, "Organic Chemistry");
    assert_eq!(updated.color.as_deref(), Some("#22c55e"));
    assert_eq!(updated.position, 5);
    assert_eq!(updated.created_at, created.created_at);
    assert!(updated.updated_at >= created.updated_at);
}

#[test]
fn soft_delete_hides_a_subject_from_list_but_get_still_finds_it() {
    let db = new_db();
    let repo = SubjectRepo::new(&db);
    let a = repo.create("Keep", None, None, 0).unwrap();
    let b = repo.create("Remove", None, None, 1).unwrap();

    repo.soft_delete(&b.id).unwrap();

    let listed: Vec<String> = repo.list().unwrap().into_iter().map(|s| s.id).collect();
    assert_eq!(listed, vec![a.id]);

    let still_there = repo.get(&b.id).unwrap().unwrap();
    assert!(still_there.deleted_at.is_some());
}

// --- PresetRepo --------------------------------------------------------

fn pomodoro_preset(subject_id: Option<&str>) -> NewPreset<'_> {
    NewPreset {
        subject_id,
        name: "Classic Pomodoro",
        mode: "pomodoro",
        work_seconds: 25 * 60,
        break_seconds: Some(5 * 60),
        long_break_seconds: Some(15 * 60),
        cycles_before_long: Some(4),
        auto_start_next: true,
        is_default: true,
    }
}

#[test]
fn a_created_preset_can_be_global_with_no_subject() {
    let db = new_db();
    let repo = PresetRepo::new(&db);

    let preset = repo.create(pomodoro_preset(None)).unwrap();

    assert_eq!(preset.subject_id, None);
    assert_eq!(preset.work_seconds, 25 * 60);
    assert!(preset.auto_start_next);
    assert!(preset.is_default);
}

#[test]
fn a_preset_scoped_to_a_subject_round_trips_through_get() {
    let db = new_db();
    let subject = SubjectRepo::new(&db).create("Math", None, None, 0).unwrap();
    let repo = PresetRepo::new(&db);
    let created = repo.create(pomodoro_preset(Some(&subject.id))).unwrap();

    let found = repo.get(&created.id).unwrap().unwrap();

    assert_eq!(found.subject_id.as_deref(), Some(subject.id.as_str()));
}

#[test]
fn soft_deleted_presets_are_excluded_from_list() {
    let db = new_db();
    let repo = PresetRepo::new(&db);
    let kept = repo.create(pomodoro_preset(None)).unwrap();
    let removed = repo.create(pomodoro_preset(None)).unwrap();

    repo.soft_delete(&removed.id).unwrap();

    let listed: Vec<String> = repo.list().unwrap().into_iter().map(|p| p.id).collect();
    assert_eq!(listed, vec![kept.id]);
}

#[test]
fn update_replaces_every_field() {
    let db = new_db();
    let repo = PresetRepo::new(&db);
    let created = repo.create(pomodoro_preset(None)).unwrap();

    repo.update(
        &created.id,
        NewPreset {
            subject_id: None,
            name: "Deep Work",
            mode: "countdown",
            work_seconds: 50 * 60,
            break_seconds: None,
            long_break_seconds: None,
            cycles_before_long: None,
            auto_start_next: false,
            is_default: false,
        },
    )
    .unwrap();

    let updated = repo.get(&created.id).unwrap().unwrap();
    assert_eq!(updated.name, "Deep Work");
    assert_eq!(updated.mode, "countdown");
    assert_eq!(updated.work_seconds, 50 * 60);
    assert_eq!(updated.break_seconds, None);
    assert!(!updated.auto_start_next);
}

// --- SessionRepo -------------------------------------------------------

#[test]
fn creating_a_session_requires_an_existing_subject() {
    let db = new_db();
    let repo = SessionRepo::new(&db);
    let started = Utc::now();

    let result = repo.create(NewSession {
        subject_id: "no-such-subject",
        preset_id: None,
        mode: "pomodoro",
        phase: "work",
        started_at: started,
        ended_at: started + TimeDelta::minutes(25),
        day_key: "2026-08-06",
        active_seconds: 1500,
        paused_seconds: 0,
        planned_seconds: Some(1500),
        completed: true,
        interruptions: 0,
        device_id: None,
    });

    assert!(
        result.is_err(),
        "foreign_keys=ON should reject a session for a subject that does not exist"
    );
}

#[test]
fn list_for_day_and_list_for_subject_return_recorded_sessions() {
    let db = new_db();
    let subject = SubjectRepo::new(&db).create("Math", None, None, 0).unwrap();
    let repo = SessionRepo::new(&db);
    let started = Utc::now();

    let session = repo
        .create(NewSession {
            subject_id: &subject.id,
            preset_id: None,
            mode: "pomodoro",
            phase: "work",
            started_at: started,
            ended_at: started + TimeDelta::minutes(25),
            day_key: "2026-08-06",
            active_seconds: 1500,
            paused_seconds: 0,
            planned_seconds: Some(1500),
            completed: true,
            interruptions: 1,
            device_id: Some("test-device"),
        })
        .unwrap();

    assert_eq!(
        repo.list_for_day("2026-08-06").unwrap(),
        vec![session.clone()]
    );
    assert_eq!(repo.list_for_day("2026-08-07").unwrap(), vec![]);
    assert_eq!(repo.list_for_subject(&subject.id).unwrap(), vec![session]);
}

// --- SettingsRepo ----------------------------------------------------------

#[test]
fn a_setting_that_was_never_written_reads_as_none() {
    let db = new_db();

    assert_eq!(SettingsRepo::new(&db).get("zen.font_size").unwrap(), None);
}

#[test]
fn setting_the_same_key_twice_replaces_the_value_and_keeps_one_row() {
    let db = new_db();
    let repo = SettingsRepo::new(&db);

    repo.set("zen.font_size", "large").unwrap();
    repo.set("zen.font_size", "huge").unwrap();

    assert_eq!(repo.get("zen.font_size").unwrap().as_deref(), Some("huge"));
    assert_eq!(repo.all().unwrap().len(), 1);
}

#[test]
fn all_returns_every_setting_sorted_by_key() {
    let db = new_db();
    let repo = SettingsRepo::new(&db);

    repo.set("zen.minutes_only", "1").unwrap();
    repo.set("day.start_offset_seconds", "14400").unwrap();

    assert_eq!(
        repo.all().unwrap(),
        vec![
            ("day.start_offset_seconds".to_string(), "14400".to_string()),
            ("zen.minutes_only".to_string(), "1".to_string()),
        ]
    );
}

// --- DeckRepo и CardRepo ---------------------------------------------------

/// Колода, привязанная к предмету, и предмет под неё.
fn deck_with_subject(db: &Database) -> (String, String) {
    let subject = SubjectRepo::new(db)
        .create("Математический анализ", Some("subject-1"), None, 0)
        .unwrap();
    let deck = DeckRepo::new(db)
        .create(Some(&subject.id), "Терсенов, § 25 — § 40", Some("лекции"))
        .unwrap();

    (subject.id, deck.id)
}

#[test]
fn a_created_deck_comes_back_with_its_subject_and_description() {
    let db = new_db();
    let (subject_id, deck_id) = deck_with_subject(&db);

    let deck = DeckRepo::new(&db).get(&deck_id).unwrap().unwrap();

    assert_eq!(deck.subject_id.as_deref(), Some(subject_id.as_str()));
    assert_eq!(deck.name, "Терсенов, § 25 — § 40");
    assert_eq!(deck.description.as_deref(), Some("лекции"));
    assert_eq!(deck.deleted_at, None);
}

#[test]
fn a_deck_can_belong_to_no_subject_at_all() {
    let db = new_db();

    let deck = DeckRepo::new(&db).create(None, "Разное", None).unwrap();

    assert_eq!(deck.subject_id, None);
    assert_eq!(deck.description, None);
}

#[test]
fn a_soft_deleted_deck_leaves_the_list_but_stays_findable() {
    let db = new_db();
    let repo = DeckRepo::new(&db);
    let deck = repo.create(None, "Разное", None).unwrap();

    repo.soft_delete(&deck.id).unwrap();

    assert!(repo.list().unwrap().is_empty());
    assert!(repo.get(&deck.id).unwrap().unwrap().deleted_at.is_some());
}

#[test]
fn cards_are_listed_for_their_own_deck_in_the_order_they_arrived() {
    let db = new_db();
    let (_, deck_id) = deck_with_subject(&db);
    let other = DeckRepo::new(&db).create(None, "Другая", None).unwrap();
    let repo = CardRepo::new(&db);

    repo.create(NewCard {
        deck_id: &deck_id,
        front: "Первообразная",
        back: "$F'=f$",
        hint: None,
        tags: Some("определение"),
    })
    .unwrap();
    repo.create(NewCard {
        deck_id: &deck_id,
        front: "Интеграл",
        back: "множество первообразных",
        hint: Some("подсказка"),
        tags: None,
    })
    .unwrap();
    repo.create(NewCard {
        deck_id: &other.id,
        front: "Чужая",
        back: "карточка",
        hint: None,
        tags: None,
    })
    .unwrap();

    let cards = repo.list_for_deck(&deck_id).unwrap();

    assert_eq!(cards.len(), 2);
    assert_eq!(cards[0].front, "Первообразная");
    assert_eq!(cards[0].tags.as_deref(), Some("определение"));
    assert_eq!(cards[1].hint.as_deref(), Some("подсказка"));
}

#[test]
fn an_import_writes_every_card_at_once() {
    let db = new_db();
    let (_, deck_id) = deck_with_subject(&db);
    let cards: Vec<NewCard<'_>> = (0..143)
        .map(|_| NewCard {
            deck_id: &deck_id,
            front: "Лицо",
            back: "Оборот",
            hint: None,
            tags: None,
        })
        .collect();

    let written = CardRepo::new(&db).create_many(&cards).unwrap();

    assert_eq!(written, 143);
    assert_eq!(
        CardRepo::new(&db).list_for_deck(&deck_id).unwrap().len(),
        143
    );
}

#[test]
fn a_failed_import_leaves_the_deck_as_it_was() {
    // Половина импорта хуже, чем ничего: понять, какая именно половина
    // доехала, потом уже нельзя.
    let db = new_db();
    let (_, deck_id) = deck_with_subject(&db);
    let repo = CardRepo::new(&db);
    let cards = vec![
        NewCard {
            deck_id: &deck_id,
            front: "Хорошая",
            back: "карточка",
            hint: None,
            tags: None,
        },
        NewCard {
            deck_id: "колоды с таким id нет",
            front: "Плохая",
            back: "карточка",
            hint: None,
            tags: None,
        },
    ];

    assert!(repo.create_many(&cards).is_err());
    assert!(repo.list_for_deck(&deck_id).unwrap().is_empty());
}

#[test]
fn editing_a_card_replaces_its_sides_hint_and_tags() {
    let db = new_db();
    let (_, deck_id) = deck_with_subject(&db);
    let repo = CardRepo::new(&db);
    let card = repo
        .create(NewCard {
            deck_id: &deck_id,
            front: "Было",
            back: "старое",
            hint: Some("подсказка"),
            tags: Some("тег"),
        })
        .unwrap();

    repo.update(&card.id, "Стало", "новое", None, Some("тег,ещё"))
        .unwrap();

    let updated = repo.get(&card.id).unwrap().unwrap();
    assert_eq!(updated.front, "Стало");
    assert_eq!(updated.back, "новое");
    assert_eq!(updated.hint, None);
    assert_eq!(updated.tags.as_deref(), Some("тег,ещё"));
    assert!(updated.updated_at >= updated.created_at);
}

#[test]
fn a_card_can_be_moved_to_another_deck() {
    let db = new_db();
    let (_, deck_id) = deck_with_subject(&db);
    let other = DeckRepo::new(&db).create(None, "Другая", None).unwrap();
    let repo = CardRepo::new(&db);
    let card = repo
        .create(NewCard {
            deck_id: &deck_id,
            front: "Карточка",
            back: "оборот",
            hint: None,
            tags: None,
        })
        .unwrap();

    repo.move_to_deck(&card.id, &other.id).unwrap();

    assert!(repo.list_for_deck(&deck_id).unwrap().is_empty());
    assert_eq!(repo.list_for_deck(&other.id).unwrap().len(), 1);
}

#[test]
fn a_soft_deleted_card_stops_being_listed_and_stops_being_counted() {
    let db = new_db();
    let (_, deck_id) = deck_with_subject(&db);
    let repo = CardRepo::new(&db);
    let card = repo
        .create(NewCard {
            deck_id: &deck_id,
            front: "Карточка",
            back: "оборот",
            hint: None,
            tags: None,
        })
        .unwrap();

    repo.soft_delete(&card.id).unwrap();

    assert!(repo.list_for_deck(&deck_id).unwrap().is_empty());
    assert!(repo.get(&card.id).unwrap().unwrap().deleted_at.is_some());
    assert_eq!(
        DeckRepo::new(&db).card_counts().unwrap(),
        vec![(deck_id, 0)]
    );
}

#[test]
fn every_deck_is_counted_including_the_empty_ones() {
    let db = new_db();
    let repo = DeckRepo::new(&db);
    let full = repo.create(None, "С карточками", None).unwrap();
    let empty = repo.create(None, "Пустая", None).unwrap();
    CardRepo::new(&db)
        .create(NewCard {
            deck_id: &full.id,
            front: "Лицо",
            back: "Оборот",
            hint: None,
            tags: None,
        })
        .unwrap();

    let mut counts = repo.card_counts().unwrap();
    counts.sort();
    let mut expected = vec![(full.id, 1), (empty.id, 0)];
    expected.sort();

    assert_eq!(counts, expected);
}

// --- ReviewRepo ------------------------------------------------------------

/// Карточка, к которой можно писать ответы.
fn card_to_review(db: &Database) -> String {
    let deck = DeckRepo::new(db).create(None, "Колода", None).unwrap();
    CardRepo::new(db)
        .create(NewCard {
            deck_id: &deck.id,
            front: "Лицо",
            back: "Оборот",
            hint: None,
            tags: None,
        })
        .unwrap()
        .id
}

fn review(db: &Database, card_id: &str, day: &str, result: &str, correct: bool) {
    ReviewRepo::new(db)
        .create(NewReview {
            card_id,
            reviewed_at: Utc::now(),
            day_key: day,
            result,
            correct,
            mode: "classic",
            think_ms: Some(4_000),
            total_ms: Some(9_000),
            device_id: None,
        })
        .unwrap();
}

#[test]
fn an_answer_comes_back_with_everything_it_was_given() {
    let db = new_db();
    let card = card_to_review(&db);

    review(&db, &card, "2026-08-21", "hard", true);

    let stored = ReviewRepo::new(&db).list_for_card(&card).unwrap();
    assert_eq!(stored.len(), 1);
    assert_eq!(stored[0].result, "hard");
    assert!(stored[0].correct);
    assert_eq!(stored[0].mode, "classic");
    assert_eq!(stored[0].think_ms, Some(4_000));
    assert_eq!(stored[0].total_ms, Some(9_000));
}

#[test]
fn answers_are_listed_per_card_and_per_day() {
    let db = new_db();
    let one = card_to_review(&db);
    let two = card_to_review(&db);

    review(&db, &one, "2026-08-21", "good", true);
    review(&db, &one, "2026-08-22", "again", false);
    review(&db, &two, "2026-08-21", "easy", true);

    assert_eq!(ReviewRepo::new(&db).list_for_card(&one).unwrap().len(), 2);
    assert_eq!(
        ReviewRepo::new(&db)
            .list_for_day("2026-08-21")
            .unwrap()
            .len(),
        2
    );
}

#[test]
fn an_answer_to_a_card_that_does_not_exist_is_refused_by_the_schema() {
    // Внешние ключи включены, и это защищает историю от мусора.
    let db = new_db();

    assert!(ReviewRepo::new(&db)
        .create(NewReview {
            card_id: "нет такой карточки",
            reviewed_at: Utc::now(),
            day_key: "2026-08-21",
            result: "good",
            correct: true,
            mode: "classic",
            think_ms: None,
            total_ms: None,
            device_id: None,
        })
        .is_err());
}
