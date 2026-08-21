//! Settings the screens read and write.
//!
//! Only the black screen's settings live here today; the study day boundary
//! joins them in M8. The pair is read and written as a whole rather than key
//! by key — the settings screen has both controls on it, and one round trip
//! is simpler than two.

use tauri::State;

use crate::core::settings::{ZenFontSize, ZenSettings};
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
) -> Result<ZenSettings, CommandError> {
    let settings = ZenSettings {
        minutes_only,
        font_size: ZenFontSize::parse(font_size)?,
    };

    let repo = SettingsRepo::new(db);
    for (key, value) in settings.to_pairs() {
        repo.set(key, &value)?;
    }

    Ok(settings)
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
) -> Result<ZenSettings, CommandError> {
    write_zen(&db, minutes_only, &font_size)
}
