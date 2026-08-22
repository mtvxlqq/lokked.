//! The streak page: days in a row, the record, the freezes, the calendar and
//! the milestones ahead.
//!
//! One command, one round trip: the page is a single screen and every number
//! on it comes from the same walk over the same days, so splitting it into
//! four commands would only give the screen four chances to disagree with
//! itself.
//!
//! Nothing is counted here — [`crate::core::stats::streak`] does the walking.
//! This layer reads the settings, asks the database for the days, and hands
//! the result over.
//!
//! The share image is the one thing that leaves the app: the frontend draws
//! it on a `<canvas>` and hands the data URL back, and [`save_image`] writes
//! it into the pictures directory. No save dialog — that is a platform
//! permission of its own, and the file is easier to find in «Изображения»
//! than in whatever folder a dialog last remembered.

use std::path::PathBuf;

use chrono::{Datelike, NaiveDate};
use serde::Serialize;
use tauri::{AppHandle, Manager, State};

use crate::core::base64::decode;
use crate::core::settings::StreakSettings;
use crate::core::stats::streak::{
    milestones, month_days, streak_state, DayMark, Milestone, StreakState, FREEZE_EVERY_DAYS,
    MAX_FREEZES, STREAK_WINDOW_DAYS,
};
use crate::core::stats::time::shift_day;
use crate::db::sessions::SessionRepo;
use crate::db::settings::SettingsRepo;
use crate::db::Database;

use super::settings::read_day;
use super::stats::today;
use super::CommandError;

/// One calendar month of the streak, as the page draws it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct MonthView {
    pub year: i32,
    /// 1–12.
    pub month: u32,
    pub days: Vec<DayMark>,
}

/// Everything the streak page shows.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct StreakView {
    /// Which study day it is, by the student's own boundary.
    pub today: String,
    /// How much has been studied today, and how much makes it count.
    pub today_seconds: i64,
    pub min_seconds: i64,
    /// Where the study day starts, so the page can say so out loud.
    pub day_start_seconds: i64,
    pub current: u32,
    pub longest: u32,
    pub longest_from: Option<String>,
    pub longest_to: Option<String>,
    pub freezes: u32,
    pub max_freezes: u32,
    /// How many days in a row earn one more freeze.
    pub freeze_every: u32,
    /// Freezes spent inside the current streak.
    pub frozen_days: u32,
    pub milestones: Vec<Milestone>,
    pub month: MonthView,
}

/// Reads the daily minimum out of the settings table.
pub fn read_streak(db: &Database) -> Result<StreakSettings, CommandError> {
    let stored = SettingsRepo::new(db).all()?;

    Ok(StreakSettings::from_pairs(
        stored
            .iter()
            .map(|(key, value)| (key.as_str(), value.as_str())),
    ))
}

/// Validates and stores the daily minimum.
pub fn write_streak(db: &Database, min_seconds: i64) -> Result<StreakSettings, CommandError> {
    let settings = StreakSettings::new(min_seconds)?;

    let repo = SettingsRepo::new(db);
    for (key, value) in settings.to_pairs() {
        repo.set(key, &value)?;
    }

    Ok(settings)
}

/// The page as of the study day `today`.
pub fn streak_page(db: &Database, today: &str) -> Result<StreakView, CommandError> {
    let settings = read_streak(db)?;
    let window = shift_day(today, -STREAK_WINDOW_DAYS);
    let days = SessionRepo::new(db).active_seconds_by_day(&window, today)?;

    let state = streak_state(&days, today, settings.rules());
    let (year, month) = year_and_month(today);

    Ok(StreakView {
        today: today.to_string(),
        today_seconds: seconds_on(&state, today),
        min_seconds: settings.min_seconds,
        day_start_seconds: read_day(db)?.start_offset_seconds,
        current: state.current,
        longest: state.longest,
        longest_from: state.longest_from.clone(),
        longest_to: state.longest_to.clone(),
        freezes: state.freezes,
        max_freezes: MAX_FREEZES,
        freeze_every: FREEZE_EVERY_DAYS,
        frozen_days: state.frozen_days,
        milestones: milestones(&state),
        month: MonthView {
            year,
            month,
            days: month_days(&state, today, year, month),
        },
    })
}

/// How much was studied on one day, or nothing if the walk never reached it.
fn seconds_on(state: &StreakState, day: &str) -> i64 {
    state
        .days
        .iter()
        .find(|marked| marked.day == day)
        .map_or(0, |marked| marked.seconds)
}

/// Which month `today` falls in, defaulting to the calendar month when the
/// day key is unreadable — the page still has to draw something.
fn year_and_month(today: &str) -> (i32, u32) {
    let date = NaiveDate::parse_from_str(today, "%Y-%m-%d")
        .unwrap_or_else(|_| chrono::Local::now().date_naive());

    (date.year(), date.month())
}

/// Writes the share image next to the student's other pictures.
///
/// `png` is what `canvas.toDataURL('image/png')` returned, prefix and all.
/// Returns the path written, because the only thing the screen can usefully
/// say afterwards is where the file went.
pub fn save_image(directory: PathBuf, png: &str, today: &str) -> Result<String, CommandError> {
    let bytes = decode(png).map_err(|err| CommandError {
        kind: super::ErrorKind::Validation,
        message: err.to_string(),
    })?;

    if bytes.is_empty() {
        return Err(CommandError {
            kind: super::ErrorKind::Validation,
            message: "картинку нечего сохранять".to_string(),
        });
    }

    let path = directory.join(format!("lokked-streak-{today}.png"));
    std::fs::create_dir_all(&directory)
        .and_then(|()| std::fs::write(&path, &bytes))
        .map_err(|err| CommandError {
            kind: super::ErrorKind::Database,
            message: format!("картинка не записалась: {err}"),
        })?;

    Ok(path.to_string_lossy().into_owned())
}

#[tauri::command]
pub fn streak_save_image(
    app: AppHandle,
    db: State<'_, Database>,
    png: String,
) -> Result<String, CommandError> {
    let today = today(&db)?;
    // «Изображения», а если система такой папки не знает — рядом с базой.
    let directory = app
        .path()
        .picture_dir()
        .or_else(|_| app.path().app_data_dir())
        .map_err(|err| CommandError {
            kind: super::ErrorKind::Database,
            message: format!("некуда сохранить картинку: {err}"),
        })?;

    save_image(directory, &png, &today)
}

#[tauri::command]
pub fn streak_view(db: State<'_, Database>) -> Result<StreakView, CommandError> {
    let today = today(&db)?;

    streak_page(&db, &today)
}

#[tauri::command]
pub fn streak_settings(db: State<'_, Database>) -> Result<StreakSettings, CommandError> {
    read_streak(&db)
}

#[tauri::command]
pub fn set_streak_settings(
    db: State<'_, Database>,
    min_seconds: i64,
) -> Result<StreakSettings, CommandError> {
    write_streak(&db, min_seconds)
}
