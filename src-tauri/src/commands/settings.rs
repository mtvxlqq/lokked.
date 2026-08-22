//! Settings the screens read and write: the black screen's look, and where
//! the study day begins.
//!
//! A group is read and written as a whole rather than key by key — the
//! settings screen shows a group at a time, and one round trip per group is
//! simpler than one per control.

use chrono::TimeDelta;
use tauri::State;

use crate::core::settings::{
    AdaptiveSettings, BlitzSettings, DaySettings, ZenFontSize, ZenSettings,
};
use crate::db::settings::SettingsRepo;
use crate::db::Database;

use super::CommandError;

/// The black screen's settings as they are stored.
pub fn read_zen(db: &Database) -> Result<ZenSettings, CommandError> {
    let stored = SettingsRepo::new(db).all()?;

    Ok(ZenSettings::from_pairs(
        stored
            .iter()
            .map(|(key, value)| (key.as_str(), value.as_str())),
    ))
}

/// Validates and stores the black screen's settings, returning what is now
/// in the table — the screen renders the answer rather than its own guess.
pub fn write_zen(
    db: &Database,
    minutes_only: bool,
    font_size: &str,
    dim_when_idle: bool,
) -> Result<ZenSettings, CommandError> {
    let settings = ZenSettings {
        minutes_only,
        font_size: ZenFontSize::parse(font_size)?,
        dim_when_idle,
    };

    let repo = SettingsRepo::new(db);
    for (key, value) in settings.to_pairs() {
        repo.set(key, &value)?;
    }

    Ok(settings)
}

/// Where the study day starts, as it is stored.
pub fn read_day(db: &Database) -> Result<DaySettings, CommandError> {
    let stored = SettingsRepo::new(db).all()?;

    Ok(DaySettings::from_pairs(
        stored
            .iter()
            .map(|(key, value)| (key.as_str(), value.as_str())),
    ))
}

/// Validates and stores the study day boundary.
pub fn write_day(db: &Database, start_offset_seconds: i64) -> Result<DaySettings, CommandError> {
    let settings = DaySettings::new(start_offset_seconds)?;

    let repo = SettingsRepo::new(db);
    for (key, value) in settings.to_pairs() {
        repo.set(key, &value)?;
    }

    Ok(settings)
}

/// The day boundary in the form [`crate::core::dayline`] takes.
///
/// Every command that files something against a study day goes through this,
/// so the boundary is read from the table in one place rather than being
/// passed around as a constant that could drift.
pub fn day_start(db: &Database) -> Result<TimeDelta, CommandError> {
    Ok(read_day(db)?.start_offset())
}

#[tauri::command]
pub fn zen_settings(db: State<'_, Database>) -> Result<ZenSettings, CommandError> {
    read_zen(&db)
}

#[tauri::command]
pub fn set_zen_settings(
    db: State<'_, Database>,
    minutes_only: bool,
    font_size: String,
    dim_when_idle: bool,
) -> Result<ZenSettings, CommandError> {
    write_zen(&db, minutes_only, &font_size, dim_when_idle)
}

/// How long a blitz card lasts, as it is stored.
pub fn read_blitz(db: &Database) -> Result<BlitzSettings, CommandError> {
    let stored = SettingsRepo::new(db).all()?;

    Ok(BlitzSettings::from_pairs(
        stored
            .iter()
            .map(|(key, value)| (key.as_str(), value.as_str())),
    ))
}

/// Validates and stores the blitz card time.
pub fn write_blitz(db: &Database, seconds: i64) -> Result<BlitzSettings, CommandError> {
    let settings = BlitzSettings::new(seconds)?;

    let repo = SettingsRepo::new(db);
    for (key, value) in settings.to_pairs() {
        repo.set(key, &value)?;
    }

    Ok(settings)
}

/// The blitz card time in seconds — what [`crate::commands::study`] arms the
/// deadline with.
pub fn blitz_seconds(db: &Database) -> Result<i64, CommandError> {
    Ok(read_blitz(db)?.seconds)
}

#[tauri::command]
pub fn blitz_settings(db: State<'_, Database>) -> Result<BlitzSettings, CommandError> {
    read_blitz(&db)
}

#[tauri::command]
pub fn set_blitz_settings(
    db: State<'_, Database>,
    seconds: i64,
) -> Result<BlitzSettings, CommandError> {
    write_blitz(&db, seconds)
}

/// How sharply the picker leans towards the weak cards, as it is stored.
pub fn read_adaptive(db: &Database) -> Result<AdaptiveSettings, CommandError> {
    let stored = SettingsRepo::new(db).all()?;

    Ok(AdaptiveSettings::from_pairs(
        stored
            .iter()
            .map(|(key, value)| (key.as_str(), value.as_str())),
    ))
}

/// Validates and stores the lean towards the weak cards.
pub fn write_adaptive(
    db: &Database,
    aggressiveness: i64,
) -> Result<AdaptiveSettings, CommandError> {
    let settings = AdaptiveSettings::new(aggressiveness)?;

    let repo = SettingsRepo::new(db);
    for (key, value) in settings.to_pairs() {
        repo.set(key, &value)?;
    }

    Ok(settings)
}

/// The exponent [`crate::commands::study`] weighs a deck with.
pub fn adaptive_exponent(db: &Database) -> Result<f64, CommandError> {
    Ok(read_adaptive(db)?.exponent())
}

#[tauri::command]
pub fn adaptive_settings(db: State<'_, Database>) -> Result<AdaptiveSettings, CommandError> {
    read_adaptive(&db)
}

#[tauri::command]
pub fn set_adaptive_settings(
    db: State<'_, Database>,
    aggressiveness: i64,
) -> Result<AdaptiveSettings, CommandError> {
    write_adaptive(&db, aggressiveness)
}

#[tauri::command]
pub fn day_settings(db: State<'_, Database>) -> Result<DaySettings, CommandError> {
    read_day(&db)
}

#[tauri::command]
pub fn set_day_settings(
    db: State<'_, Database>,
    start_offset_seconds: i64,
) -> Result<DaySettings, CommandError> {
    write_day(&db, start_offset_seconds)
}
