//! CRUD tests for `SubjectRepo`, `PresetRepo`, `SessionRepo` and
//! `SettingsRepo`, plus the
//! cross-cutting behaviours the schema promises: soft delete hides rows from
//! `list` but not `get`, and foreign keys are actually enforced.
//!
//! Schema-application tests live in `tests/db_migrations.rs`.

use chrono::{TimeDelta, Utc};
use lokked_lib::db::presets::{NewPreset, PresetRepo};
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
