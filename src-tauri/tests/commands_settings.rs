//! Tests for the settings commands: what an untouched install reports, what
//! a save stores, and what happens to a value the settings screen should
//! never have sent. Both groups live in one table, so they are also tested
//! for staying out of each other's way.

use chrono::TimeDelta;
use lokked_lib::commands::settings::{day_start, read_day, read_zen, write_day, write_zen};
use lokked_lib::commands::ErrorKind;
use lokked_lib::core::settings::{
    DaySettings, ZenFontSize, ZenSettings, KEY_DAY_START, KEY_DIM_WHEN_IDLE, KEY_FONT_SIZE,
    KEY_MINUTES_ONLY,
};
use lokked_lib::db::settings::SettingsRepo;
use lokked_lib::db::Database;

fn new_db() -> Database {
    Database::open_in_memory().expect("in-memory database should open")
}

#[test]
fn an_untouched_install_reports_the_defaults() {
    let db = new_db();

    assert_eq!(read_zen(&db).unwrap(), ZenSettings::default());
}

#[test]
fn what_was_saved_is_what_is_read_back() {
    let db = new_db();

    let saved = write_zen(&db, true, "large", false).unwrap();
    let read = read_zen(&db).unwrap();

    assert_eq!(
        saved,
        ZenSettings {
            minutes_only: true,
            font_size: ZenFontSize::Large,
            dim_when_idle: false,
        }
    );
    assert_eq!(read, saved);
}

#[test]
fn saving_twice_replaces_the_stored_rows_instead_of_piling_them_up() {
    let db = new_db();

    write_zen(&db, true, "small", false).unwrap();
    write_zen(&db, false, "normal", true).unwrap();

    assert_eq!(read_zen(&db).unwrap(), ZenSettings::default());
    assert_eq!(SettingsRepo::new(&db).all().unwrap().len(), 3);
}

#[test]
fn the_stored_rows_use_the_documented_keys_and_values() {
    // The keys are part of the database's vocabulary, not an implementation
    // detail: a future sync merges rows by them.
    let db = new_db();
    let repo = SettingsRepo::new(&db);

    write_zen(&db, true, "large", false).unwrap();

    assert_eq!(repo.get(KEY_MINUTES_ONLY).unwrap().as_deref(), Some("1"));
    assert_eq!(repo.get(KEY_FONT_SIZE).unwrap().as_deref(), Some("large"));
    assert_eq!(repo.get(KEY_DIM_WHEN_IDLE).unwrap().as_deref(), Some("0"));
}

#[test]
fn an_unknown_font_size_is_rejected_and_nothing_is_written() {
    let db = new_db();

    let error = write_zen(&db, true, "gigantic", true).unwrap_err();

    assert_eq!(error.kind, ErrorKind::Validation);
    assert!(error.message.contains("gigantic"));
    assert!(SettingsRepo::new(&db).all().unwrap().is_empty());
}

#[test]
fn a_value_left_over_from_another_version_does_not_break_reading() {
    let db = new_db();
    let repo = SettingsRepo::new(&db);

    repo.set(KEY_FONT_SIZE, "gigantic").unwrap();
    repo.set(KEY_MINUTES_ONLY, "1").unwrap();

    assert_eq!(
        read_zen(&db).unwrap(),
        ZenSettings {
            minutes_only: true,
            font_size: ZenFontSize::Normal,
            dim_when_idle: true,
        }
    );
}

#[test]
fn an_unknown_setting_is_left_alone_by_a_save() {
    // The table is shared with settings other screens own; writing the black
    // screen's pair must not disturb them.
    let db = new_db();
    let repo = SettingsRepo::new(&db);
    repo.set("day.start_offset_seconds", "14400").unwrap();

    write_zen(&db, true, "large", true).unwrap();

    assert_eq!(
        repo.get("day.start_offset_seconds").unwrap().as_deref(),
        Some("14400")
    );
}

// --- граница учебного дня --------------------------------------------------

#[test]
fn dimming_stays_on_until_it_is_switched_off() {
    // Апдейт не должен менять поведение экрана у того, кто ничего не трогал.
    let db = new_db();

    assert!(read_zen(&db).unwrap().dim_when_idle);

    write_zen(&db, false, "normal", false).unwrap();

    assert!(!read_zen(&db).unwrap().dim_when_idle);
}

#[test]
fn the_study_day_starts_at_midnight_until_it_is_changed() {
    let db = new_db();

    assert_eq!(read_day(&db).unwrap(), DaySettings::default());
    assert_eq!(day_start(&db).unwrap(), TimeDelta::zero());
}

#[test]
fn a_saved_boundary_is_what_the_rest_of_the_app_reads() {
    let db = new_db();

    write_day(&db, 4 * 60 * 60).unwrap();

    assert_eq!(day_start(&db).unwrap(), TimeDelta::hours(4));
    assert_eq!(
        SettingsRepo::new(&db)
            .get(KEY_DAY_START)
            .unwrap()
            .as_deref(),
        Some("14400")
    );
}

#[test]
fn a_boundary_outside_the_day_is_rejected_and_nothing_is_written() {
    let db = new_db();

    let error = write_day(&db, 25 * 60 * 60).unwrap_err();

    assert_eq!(error.kind, ErrorKind::Validation);
    assert!(SettingsRepo::new(&db).all().unwrap().is_empty());
}

#[test]
fn the_two_groups_of_settings_do_not_overwrite_each_other() {
    let db = new_db();

    write_zen(&db, true, "large", false).unwrap();
    write_day(&db, 3 * 60 * 60).unwrap();

    assert_eq!(day_start(&db).unwrap(), TimeDelta::hours(3));
    assert_eq!(
        read_zen(&db).unwrap(),
        ZenSettings {
            minutes_only: true,
            font_size: ZenFontSize::Large,
            dim_when_idle: false,
        }
    );
}
