//! Tests for the backup that runs at startup: that the copy is a real,
//! complete database, and that old copies are rotated out.

use std::fs;
use std::path::PathBuf;

use chrono::{TimeDelta, TimeZone, Utc};
use lokked_lib::core::backup::backup_name;
use lokked_lib::db::backup::{prune, rotate};
use lokked_lib::db::subjects::SubjectRepo;
use lokked_lib::db::Database;
use uuid::Uuid;

/// A directory of its own per test, removed when the test ends.
struct TempDir(PathBuf);

impl TempDir {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!("lokked-backup-{}", Uuid::now_v7()));
        fs::create_dir_all(&path).expect("temp dir should be creatable");

        Self(path)
    }

    fn path(&self) -> &std::path::Path {
        &self.0
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

/// A file-backed database with one subject in it.
fn database(dir: &TempDir) -> Database {
    let db = Database::open_at(dir.path().join("lokked.sqlite3")).expect("database should open");
    SubjectRepo::new(&db)
        .create("Математический анализ", None, None, 0)
        .unwrap();

    db
}

fn file_names(dir: &std::path::Path) -> Vec<String> {
    let mut names: Vec<String> = fs::read_dir(dir)
        .unwrap()
        .map(|entry| entry.unwrap().file_name().into_string().unwrap())
        .collect();
    names.sort();

    names
}

#[test]
fn a_launch_leaves_a_copy_of_the_database_behind() {
    let dir = TempDir::new();
    let db = database(&dir);
    let backups = dir.path().join("backups");
    let now = Utc.with_ymd_and_hms(2026, 8, 21, 3, 45, 9).unwrap();

    let written = rotate(&db, &backups, now).unwrap();

    assert_eq!(written, backups.join(backup_name(now)));
    assert!(written.exists());
}

#[test]
fn the_copy_is_a_database_with_the_same_rows_in_it() {
    let dir = TempDir::new();
    let db = database(&dir);
    let backups = dir.path().join("backups");

    let written = rotate(&db, &backups, Utc::now()).unwrap();

    // Копию можно открыть как обычную базу, и в ней всё на месте — ровно то,
    // ради чего бэкап и делается.
    let restored = Database::open_at(&written).expect("a backup should open as a database");
    let subjects = SubjectRepo::new(&restored).list().unwrap();
    assert_eq!(subjects.len(), 1);
    assert_eq!(subjects[0].name, "Математический анализ");
}

#[test]
fn the_backup_folder_is_created_if_it_is_not_there_yet() {
    let dir = TempDir::new();
    let db = database(&dir);
    let backups = dir.path().join("backups");
    assert!(!backups.exists());

    rotate(&db, &backups, Utc::now()).unwrap();

    assert!(backups.is_dir());
}

#[test]
fn a_second_launch_within_the_same_second_does_not_fail() {
    let dir = TempDir::new();
    let db = database(&dir);
    let backups = dir.path().join("backups");
    let now = Utc.with_ymd_and_hms(2026, 8, 21, 3, 45, 9).unwrap();

    rotate(&db, &backups, now).unwrap();
    rotate(&db, &backups, now).unwrap();

    assert_eq!(file_names(&backups).len(), 1);
}

#[test]
fn only_the_last_seven_copies_survive() {
    let dir = TempDir::new();
    let db = database(&dir);
    let backups = dir.path().join("backups");
    let start = Utc.with_ymd_and_hms(2026, 8, 1, 9, 0, 0).unwrap();

    for day in 0..10 {
        rotate(&db, &backups, start + TimeDelta::days(day)).unwrap();
    }

    let names = file_names(&backups);
    assert_eq!(names.len(), 7);
    // Остались последние семь: с 4 по 10 августа.
    assert_eq!(names[0], backup_name(start + TimeDelta::days(3)));
    assert_eq!(names[6], backup_name(start + TimeDelta::days(9)));
}

#[test]
fn pruning_leaves_files_that_are_not_ours_alone() {
    let dir = TempDir::new();
    let backups = dir.path().join("backups");
    fs::create_dir_all(&backups).unwrap();
    fs::write(backups.join("заметка.txt"), "не трогать").unwrap();
    fs::write(backups.join("lokked-20260801-090000.sqlite3"), "старая").unwrap();
    fs::write(backups.join("lokked-20260802-090000.sqlite3"), "новая").unwrap();

    let removed = prune(&backups, 1).unwrap();

    assert_eq!(
        removed,
        vec![backups.join("lokked-20260801-090000.sqlite3")]
    );
    assert_eq!(
        file_names(&backups),
        vec![
            "lokked-20260802-090000.sqlite3".to_string(),
            "заметка.txt".to_string(),
        ]
    );
}
