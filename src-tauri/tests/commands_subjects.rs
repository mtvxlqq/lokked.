//! Tests for the subject commands: the rules that live between `core`
//! validation and `SubjectRepo` — default colour, appended position, and
//! refusing to touch a subject that is gone.

use lokked_lib::commands::subjects::{create, delete, list, update, SubjectInput};
use lokked_lib::commands::ErrorKind;
use lokked_lib::core::subject::PALETTE_SIZE;
use lokked_lib::db::subjects::SubjectRepo;
use lokked_lib::db::Database;

fn new_db() -> Database {
    Database::open_in_memory().expect("in-memory database should open")
}

fn input(name: &str) -> SubjectInput {
    SubjectInput {
        name: name.to_string(),
        color: None,
        icon: None,
    }
}

#[test]
fn a_new_subject_gets_the_next_palette_colour() {
    let db = new_db();

    let first = create(&db, input("Алгебра")).unwrap();
    let second = create(&db, input("Физика")).unwrap();

    assert_eq!(first.color.as_deref(), Some("subject-1"));
    assert_eq!(second.color.as_deref(), Some("subject-2"));
}

#[test]
fn the_palette_wraps_around_rather_than_running_out() {
    let db = new_db();

    for n in 0..=PALETTE_SIZE {
        let subject = create(&db, input(&format!("Предмет {n}"))).unwrap();
        let expected = format!("subject-{}", n % PALETTE_SIZE + 1);
        assert_eq!(subject.color.as_deref(), Some(expected.as_str()));
    }
}

#[test]
fn an_explicit_colour_is_kept() {
    let db = new_db();

    let subject = create(
        &db,
        SubjectInput {
            color: Some("subject-5".to_string()),
            ..input("Химия")
        },
    )
    .unwrap();

    assert_eq!(subject.color.as_deref(), Some("subject-5"));
}

#[test]
fn a_colour_outside_the_palette_is_rejected_before_anything_is_written() {
    let db = new_db();

    let error = create(
        &db,
        SubjectInput {
            color: Some("#7e9cc4".to_string()),
            ..input("Химия")
        },
    )
    .unwrap_err();

    assert_eq!(error.kind, ErrorKind::Validation);
    assert!(list(&db).unwrap().is_empty());
}

#[test]
fn a_blank_name_is_rejected() {
    let db = new_db();

    let error = create(&db, input("   ")).unwrap_err();

    assert_eq!(error.kind, ErrorKind::Validation);
    assert!(list(&db).unwrap().is_empty());
}

#[test]
fn the_name_and_icon_are_trimmed() {
    let db = new_db();

    let subject = create(
        &db,
        SubjectInput {
            icon: Some("  sigma  ".to_string()),
            ..input("  Алгебра  ")
        },
    )
    .unwrap();

    assert_eq!(subject.name, "Алгебра");
    assert_eq!(subject.icon.as_deref(), Some("sigma"));
}

#[test]
fn a_blank_icon_is_stored_as_none() {
    let db = new_db();

    let subject = create(
        &db,
        SubjectInput {
            icon: Some("   ".to_string()),
            ..input("Алгебра")
        },
    )
    .unwrap();

    assert_eq!(subject.icon, None);
}

#[test]
fn subjects_are_appended_in_order() {
    let db = new_db();

    create(&db, input("Первый")).unwrap();
    create(&db, input("Второй")).unwrap();
    create(&db, input("Третий")).unwrap();

    let positions: Vec<i64> = list(&db).unwrap().into_iter().map(|s| s.position).collect();
    assert_eq!(positions, vec![0, 1, 2]);
}

#[test]
fn a_position_freed_by_a_deleted_subject_is_not_reused() {
    let db = new_db();

    create(&db, input("Первый")).unwrap();
    let second = create(&db, input("Второй")).unwrap();
    delete(&db, &second.id).unwrap();

    // Position 1 is gone with the deleted subject; the next one takes 2, so
    // no two live subjects can end up sorting the same.
    let third = create(&db, input("Третий")).unwrap();
    assert_eq!(third.position, 2);

    let positions: Vec<i64> = list(&db).unwrap().into_iter().map(|s| s.position).collect();
    assert_eq!(positions, vec![0, 2]);
}

#[test]
fn listing_skips_deleted_subjects() {
    let db = new_db();

    let kept = create(&db, input("Алгебра")).unwrap();
    let dropped = create(&db, input("Физика")).unwrap();
    delete(&db, &dropped.id).unwrap();

    let names: Vec<String> = list(&db).unwrap().into_iter().map(|s| s.name).collect();
    assert_eq!(names, vec![kept.name]);
}

#[test]
fn updating_renames_and_recolours_without_moving_the_subject() {
    let db = new_db();

    create(&db, input("Первый")).unwrap();
    let second = create(&db, input("Второй")).unwrap();

    let updated = update(
        &db,
        second.id.clone(),
        SubjectInput {
            name: "Дискретная математика".to_string(),
            color: Some("subject-7".to_string()),
            icon: Some("sigma".to_string()),
        },
    )
    .unwrap();

    assert_eq!(updated.name, "Дискретная математика");
    assert_eq!(updated.color.as_deref(), Some("subject-7"));
    assert_eq!(updated.position, second.position);

    let stored = SubjectRepo::new(&db).get(&second.id).unwrap().unwrap();
    assert_eq!(stored.name, "Дискретная математика");
    assert_eq!(stored.color.as_deref(), Some("subject-7"));
    assert_eq!(stored.position, second.position);
}

#[test]
fn clearing_the_colour_on_update_leaves_the_subject_without_one() {
    let db = new_db();

    let subject = create(&db, input("Алгебра")).unwrap();
    let updated = update(&db, subject.id, input("Алгебра")).unwrap();

    // Unlike create, update takes the dialog at its word: an empty colour
    // means the student cleared it, not that they never chose one.
    assert_eq!(updated.color, None);
}

#[test]
fn an_invalid_update_leaves_the_stored_subject_untouched() {
    let db = new_db();

    let subject = create(&db, input("Алгебра")).unwrap();
    let error = update(&db, subject.id.clone(), input("")).unwrap_err();

    assert_eq!(error.kind, ErrorKind::Validation);
    let stored = SubjectRepo::new(&db).get(&subject.id).unwrap().unwrap();
    assert_eq!(stored.name, "Алгебра");
}

#[test]
fn updating_or_deleting_a_missing_subject_reports_not_found() {
    let db = new_db();

    let missing = "0192f0d0-0000-7000-8000-000000000000";
    assert_eq!(
        update(&db, missing.to_string(), input("Алгебра"))
            .unwrap_err()
            .kind,
        ErrorKind::NotFound
    );
    assert_eq!(delete(&db, missing).unwrap_err().kind, ErrorKind::NotFound);
}

#[test]
fn a_subject_cannot_be_deleted_twice() {
    let db = new_db();

    let subject = create(&db, input("Алгебра")).unwrap();
    delete(&db, &subject.id).unwrap();

    assert_eq!(
        delete(&db, &subject.id).unwrap_err().kind,
        ErrorKind::NotFound
    );
}

#[test]
fn a_deleted_subject_cannot_be_edited_back_into_the_list() {
    let db = new_db();

    let subject = create(&db, input("Алгебра")).unwrap();
    delete(&db, &subject.id).unwrap();

    assert_eq!(
        update(&db, subject.id, input("Алгебра")).unwrap_err().kind,
        ErrorKind::NotFound
    );
}
