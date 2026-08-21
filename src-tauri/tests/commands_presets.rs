//! Tests for the preset commands: the rules that live between `core::preset`
//! validation and `PresetRepo` — one default per scope, dropping fields the
//! chosen mode does not use, and refusing to attach a preset to a subject
//! that is gone.

use lokked_lib::commands::presets::{create, delete, list, update, PresetDto, PresetInput};
use lokked_lib::commands::subjects::{self, SubjectInput};
use lokked_lib::commands::ErrorKind;
use lokked_lib::db::presets::PresetRepo;
use lokked_lib::db::Database;

fn new_db() -> Database {
    Database::open_in_memory().expect("in-memory database should open")
}

fn subject(db: &Database, name: &str) -> String {
    subjects::create(
        db,
        SubjectInput {
            name: name.to_string(),
            color: None,
            icon: None,
        },
    )
    .unwrap()
    .id
}

/// A valid Pomodoro input; tests override the one field they are about.
fn pomodoro() -> PresetInput {
    PresetInput {
        subject_id: None,
        name: "Классический".to_string(),
        mode: "pomodoro".to_string(),
        work_seconds: 25 * 60,
        break_seconds: Some(5 * 60),
        long_break_seconds: Some(15 * 60),
        cycles_before_long: Some(4),
        auto_start_next: true,
        is_default: false,
    }
}

fn defaults(db: &Database) -> Vec<PresetDto> {
    list(db)
        .unwrap()
        .into_iter()
        .filter(|p| p.is_default)
        .collect()
}

#[test]
fn a_created_preset_comes_back_with_every_field() {
    let db = new_db();

    let preset = create(&db, pomodoro()).unwrap();

    assert!(!preset.id.is_empty());
    assert_eq!(preset.subject_id, None);
    assert_eq!(preset.name, "Классический");
    assert_eq!(preset.mode, "pomodoro");
    assert_eq!(preset.work_seconds, 25 * 60);
    assert_eq!(preset.break_seconds, Some(5 * 60));
    assert_eq!(preset.long_break_seconds, Some(15 * 60));
    assert_eq!(preset.cycles_before_long, Some(4));
    assert!(preset.auto_start_next);
    assert!(!preset.is_default);
}

#[test]
fn an_invalid_preset_is_rejected_before_anything_is_written() {
    let db = new_db();

    let error = create(
        &db,
        PresetInput {
            break_seconds: None,
            ..pomodoro()
        },
    )
    .unwrap_err();

    assert_eq!(error.kind, ErrorKind::Validation);
    assert!(list(&db).unwrap().is_empty());
}

#[test]
fn a_countdown_preset_does_not_store_the_pomodoro_fields() {
    let db = new_db();

    let preset = create(
        &db,
        PresetInput {
            mode: "countdown".to_string(),
            work_seconds: 45 * 60,
            ..pomodoro()
        },
    )
    .unwrap();

    assert_eq!(preset.work_seconds, 45 * 60);
    assert_eq!(preset.break_seconds, None);
    assert_eq!(preset.long_break_seconds, None);
    assert_eq!(preset.cycles_before_long, None);
    assert!(!preset.auto_start_next);

    // Not just in the returned DTO — the row itself must be clean.
    let stored = PresetRepo::new(&db).get(&preset.id).unwrap().unwrap();
    assert_eq!(stored.break_seconds, None);
    assert!(!stored.auto_start_next);
}

#[test]
fn switching_a_preset_to_countup_drops_its_old_durations() {
    let db = new_db();

    let preset = create(&db, pomodoro()).unwrap();
    let updated = update(
        &db,
        preset.id.clone(),
        PresetInput {
            mode: "countup".to_string(),
            ..pomodoro()
        },
    )
    .unwrap();

    assert_eq!(updated.mode, "countup");
    assert_eq!(updated.work_seconds, 0);
    assert_eq!(updated.cycles_before_long, None);

    let stored = PresetRepo::new(&db).get(&preset.id).unwrap().unwrap();
    assert_eq!(stored.work_seconds, 0);
    assert_eq!(stored.cycles_before_long, None);
    assert_eq!(stored.long_break_seconds, None);
}

#[test]
fn making_a_preset_default_demotes_the_previous_one() {
    let db = new_db();

    let first = create(
        &db,
        PresetInput {
            is_default: true,
            ..pomodoro()
        },
    )
    .unwrap();
    let second = create(
        &db,
        PresetInput {
            name: "Длинный".to_string(),
            is_default: true,
            ..pomodoro()
        },
    )
    .unwrap();

    let ids: Vec<String> = defaults(&db).into_iter().map(|p| p.id).collect();
    assert_eq!(ids, vec![second.id]);
    assert!(
        !PresetRepo::new(&db)
            .get(&first.id)
            .unwrap()
            .unwrap()
            .is_default
    );
}

#[test]
fn each_subject_keeps_its_own_default_alongside_the_global_one() {
    let db = new_db();
    let algebra = subject(&db, "Алгебра");
    let physics = subject(&db, "Физика");

    let global = create(
        &db,
        PresetInput {
            is_default: true,
            ..pomodoro()
        },
    )
    .unwrap();
    let for_algebra = create(
        &db,
        PresetInput {
            subject_id: Some(algebra),
            name: "Алгебра — длинный".to_string(),
            is_default: true,
            ..pomodoro()
        },
    )
    .unwrap();
    let for_physics = create(
        &db,
        PresetInput {
            subject_id: Some(physics),
            name: "Физика — короткий".to_string(),
            is_default: true,
            ..pomodoro()
        },
    )
    .unwrap();

    let mut ids: Vec<String> = defaults(&db).into_iter().map(|p| p.id).collect();
    ids.sort();
    let mut expected = vec![global.id, for_algebra.id, for_physics.id];
    expected.sort();
    assert_eq!(ids, expected);
}

#[test]
fn re_saving_the_default_preset_keeps_it_default() {
    let db = new_db();

    let preset = create(
        &db,
        PresetInput {
            is_default: true,
            ..pomodoro()
        },
    )
    .unwrap();

    // The dialog sends the whole preset back, default flag included: clearing
    // every other default must not clear this one's.
    let updated = update(
        &db,
        preset.id.clone(),
        PresetInput {
            name: "Классический+".to_string(),
            is_default: true,
            ..pomodoro()
        },
    )
    .unwrap();

    assert!(updated.is_default);
    let ids: Vec<String> = defaults(&db).into_iter().map(|p| p.id).collect();
    assert_eq!(ids, vec![preset.id]);
}

#[test]
fn moving_a_preset_to_another_subject_demotes_that_subjects_default() {
    let db = new_db();
    let algebra = subject(&db, "Алгебра");

    let incumbent = create(
        &db,
        PresetInput {
            subject_id: Some(algebra.clone()),
            name: "Старый".to_string(),
            is_default: true,
            ..pomodoro()
        },
    )
    .unwrap();
    let moved = create(
        &db,
        PresetInput {
            name: "Глобальный".to_string(),
            is_default: true,
            ..pomodoro()
        },
    )
    .unwrap();

    update(
        &db,
        moved.id.clone(),
        PresetInput {
            subject_id: Some(algebra),
            name: "Глобальный".to_string(),
            is_default: true,
            ..pomodoro()
        },
    )
    .unwrap();

    let ids: Vec<String> = defaults(&db).into_iter().map(|p| p.id).collect();
    assert_eq!(ids, vec![moved.id]);
    assert!(
        !PresetRepo::new(&db)
            .get(&incumbent.id)
            .unwrap()
            .unwrap()
            .is_default
    );
}

#[test]
fn a_preset_cannot_be_attached_to_a_missing_subject() {
    let db = new_db();

    let error = create(
        &db,
        PresetInput {
            subject_id: Some("0192f0d0-0000-7000-8000-000000000000".to_string()),
            ..pomodoro()
        },
    )
    .unwrap_err();

    assert_eq!(error.kind, ErrorKind::NotFound);
    assert!(list(&db).unwrap().is_empty());
}

#[test]
fn a_preset_cannot_be_attached_to_a_deleted_subject() {
    let db = new_db();
    let algebra = subject(&db, "Алгебра");
    subjects::delete(&db, &algebra).unwrap();

    let error = create(
        &db,
        PresetInput {
            subject_id: Some(algebra),
            ..pomodoro()
        },
    )
    .unwrap_err();

    assert_eq!(error.kind, ErrorKind::NotFound);
}

#[test]
fn listing_skips_deleted_presets() {
    let db = new_db();

    let kept = create(&db, pomodoro()).unwrap();
    let dropped = create(
        &db,
        PresetInput {
            name: "Лишний".to_string(),
            ..pomodoro()
        },
    )
    .unwrap();
    delete(&db, &dropped.id).unwrap();

    let ids: Vec<String> = list(&db).unwrap().into_iter().map(|p| p.id).collect();
    assert_eq!(ids, vec![kept.id]);
}

#[test]
fn updating_or_deleting_a_missing_preset_reports_not_found() {
    let db = new_db();

    let missing = "0192f0d0-0000-7000-8000-000000000000";
    assert_eq!(
        update(&db, missing.to_string(), pomodoro())
            .unwrap_err()
            .kind,
        ErrorKind::NotFound
    );
    assert_eq!(delete(&db, missing).unwrap_err().kind, ErrorKind::NotFound);
}

#[test]
fn a_deleted_preset_stays_deleted() {
    let db = new_db();

    let preset = create(&db, pomodoro()).unwrap();
    delete(&db, &preset.id).unwrap();

    assert_eq!(
        delete(&db, &preset.id).unwrap_err().kind,
        ErrorKind::NotFound
    );
    assert_eq!(
        update(&db, preset.id, pomodoro()).unwrap_err().kind,
        ErrorKind::NotFound
    );
}

#[test]
fn a_deleted_default_does_not_block_a_new_one() {
    let db = new_db();

    let old = create(
        &db,
        PresetInput {
            is_default: true,
            ..pomodoro()
        },
    )
    .unwrap();
    delete(&db, &old.id).unwrap();

    let fresh = create(
        &db,
        PresetInput {
            name: "Новый".to_string(),
            is_default: true,
            ..pomodoro()
        },
    )
    .unwrap();

    let ids: Vec<String> = defaults(&db).into_iter().map(|p| p.id).collect();
    assert_eq!(ids, vec![fresh.id]);
}
