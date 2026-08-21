//! Commands backing the timer-preset list and its create/edit dialog.

use serde::{Deserialize, Serialize};
use tauri::State;

use crate::core::preset::{validate, PresetDraft, ValidPreset};
use crate::db::presets::{NewPreset, PresetRepo, TimerPreset};
use crate::db::subjects::SubjectRepo;
use crate::db::Database;

use super::CommandError;

/// A preset as the frontend sees it.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct PresetDto {
    pub id: String,
    /// `None` means a global preset, offered for every subject.
    pub subject_id: Option<String>,
    pub name: String,
    pub mode: String,
    pub work_seconds: i64,
    pub break_seconds: Option<i64>,
    pub long_break_seconds: Option<i64>,
    pub cycles_before_long: Option<i64>,
    pub auto_start_next: bool,
    pub is_default: bool,
}

impl From<TimerPreset> for PresetDto {
    fn from(preset: TimerPreset) -> Self {
        Self {
            id: preset.id,
            subject_id: preset.subject_id,
            name: preset.name,
            mode: preset.mode,
            work_seconds: preset.work_seconds,
            break_seconds: preset.break_seconds,
            long_break_seconds: preset.long_break_seconds,
            cycles_before_long: preset.cycles_before_long,
            auto_start_next: preset.auto_start_next,
            is_default: preset.is_default,
        }
    }
}

/// What the create/edit dialog sends.
///
/// Every duration arrives as an [`Option`] regardless of mode: the dialog
/// keeps whatever the student typed while they flip between modes, and
/// [`validate`] decides which fields the chosen mode actually needs — the
/// rest are dropped rather than stored.
#[derive(Debug, Clone, Deserialize)]
pub struct PresetInput {
    pub subject_id: Option<String>,
    pub name: String,
    pub mode: String,
    #[serde(default)]
    pub work_seconds: i64,
    pub break_seconds: Option<i64>,
    pub long_break_seconds: Option<i64>,
    pub cycles_before_long: Option<i64>,
    #[serde(default)]
    pub auto_start_next: bool,
    #[serde(default)]
    pub is_default: bool,
}

impl PresetInput {
    fn validated(&self) -> Result<ValidPreset, CommandError> {
        validate(PresetDraft {
            name: &self.name,
            mode: &self.mode,
            work_seconds: self.work_seconds,
            break_seconds: self.break_seconds,
            long_break_seconds: self.long_break_seconds,
            cycles_before_long: self.cycles_before_long,
            auto_start_next: self.auto_start_next,
        })
        .map_err(CommandError::from)
    }
}

/// Builds the row to write. Takes the validated preset rather than the raw
/// input so a field the mode does not use can never reach SQLite.
fn row<'a>(valid: &'a ValidPreset, subject_id: Option<&'a str>, is_default: bool) -> NewPreset<'a> {
    NewPreset {
        subject_id,
        name: &valid.name,
        mode: valid.kind.as_str(),
        work_seconds: valid.work_seconds,
        break_seconds: valid.break_seconds,
        long_break_seconds: valid.long_break_seconds,
        cycles_before_long: valid.cycles_before_long,
        auto_start_next: valid.auto_start_next,
        is_default,
    }
}

/// Checks that a preset scoped to a subject points at one that still exists.
/// A global preset (`None`) has nothing to check.
fn check_subject(db: &Database, subject_id: Option<&str>) -> Result<(), CommandError> {
    let Some(id) = subject_id else {
        return Ok(());
    };

    SubjectRepo::new(db)
        .get(id)?
        .filter(|subject| subject.deleted_at.is_none())
        .map(|_| ())
        .ok_or_else(|| CommandError::not_found("предмет"))
}

/// Every preset that has not been deleted — both global and subject-scoped.
pub fn list(db: &Database) -> Result<Vec<PresetDto>, CommandError> {
    let presets = PresetRepo::new(db).list()?;
    Ok(presets.into_iter().map(PresetDto::from).collect())
}

/// Creates a preset, keeping only the fields its mode uses.
pub fn create(db: &Database, input: PresetInput) -> Result<PresetDto, CommandError> {
    let valid = input.validated()?;
    let subject_id = input.subject_id.as_deref();
    check_subject(db, subject_id)?;

    let repo = PresetRepo::new(db);
    // Demote the old default first: two defaults in one scope would make
    // «which preset does the Start button use» unanswerable.
    if input.is_default {
        repo.clear_default(subject_id, None)?;
    }

    let preset = repo.create(row(&valid, subject_id, input.is_default))?;
    Ok(preset.into())
}

/// Rewrites a preset. Switching its mode drops whatever the old mode used
/// and the new one does not.
pub fn update(db: &Database, id: String, input: PresetInput) -> Result<PresetDto, CommandError> {
    let valid = input.validated()?;
    let subject_id = input.subject_id.as_deref();
    check_subject(db, subject_id)?;

    let repo = PresetRepo::new(db);
    repo.get(&id)?
        .filter(|preset| preset.deleted_at.is_none())
        .ok_or_else(|| CommandError::not_found("пресет"))?;

    if input.is_default {
        repo.clear_default(subject_id, Some(&id))?;
    }

    repo.update(&id, row(&valid, subject_id, input.is_default))?;

    Ok(PresetDto {
        id,
        subject_id: input.subject_id,
        name: valid.name,
        mode: valid.kind.as_str().to_string(),
        work_seconds: valid.work_seconds,
        break_seconds: valid.break_seconds,
        long_break_seconds: valid.long_break_seconds,
        cycles_before_long: valid.cycles_before_long,
        auto_start_next: valid.auto_start_next,
        is_default: input.is_default,
    })
}

/// Soft-deletes a preset. Sessions already recorded keep pointing at it, so
/// statistics can still name the preset a session was run with.
pub fn delete(db: &Database, id: &str) -> Result<(), CommandError> {
    let repo = PresetRepo::new(db);

    repo.get(id)?
        .filter(|preset| preset.deleted_at.is_none())
        .ok_or_else(|| CommandError::not_found("пресет"))?;

    repo.soft_delete(id)?;
    Ok(())
}

// The `#[tauri::command]` wrappers — see the note in [`super::subjects`].

#[tauri::command]
pub fn list_presets(db: State<'_, Database>) -> Result<Vec<PresetDto>, CommandError> {
    list(&db)
}

#[tauri::command]
pub fn create_preset(
    db: State<'_, Database>,
    input: PresetInput,
) -> Result<PresetDto, CommandError> {
    create(&db, input)
}

#[tauri::command]
pub fn update_preset(
    db: State<'_, Database>,
    id: String,
    input: PresetInput,
) -> Result<PresetDto, CommandError> {
    update(&db, id, input)
}

#[tauri::command]
pub fn delete_preset(db: State<'_, Database>, id: String) -> Result<(), CommandError> {
    delete(&db, &id)
}
