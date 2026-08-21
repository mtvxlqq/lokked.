//! Commands backing the subject list and its create/edit dialog.

use serde::{Deserialize, Serialize};
use tauri::State;

use crate::core::subject::{normalize_color, normalize_name, palette_slug};
use crate::db::subjects::{Subject, SubjectRepo};
use crate::db::Database;

use super::CommandError;

/// A subject as the frontend sees it.
///
/// The bookkeeping columns (`created_at`, `updated_at`, `deleted_at`) stay on
/// this side of the bridge: no screen shows them, and a soft-deleted subject
/// never reaches the frontend in the first place.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct SubjectDto {
    pub id: String,
    pub name: String,
    /// A palette slug (`subject-1` … `subject-8`), never a hex colour — see
    /// [`crate::core::subject::normalize_color`].
    pub color: Option<String>,
    pub icon: Option<String>,
    pub position: i64,
}

impl From<Subject> for SubjectDto {
    fn from(subject: Subject) -> Self {
        Self {
            id: subject.id,
            name: subject.name,
            color: subject.color,
            icon: subject.icon,
            position: subject.position,
        }
    }
}

/// What the create/edit dialog sends.
///
/// `color` is optional on purpose: leaving it unset on create means «pick one
/// for me», which is what the dialog does until the student opens the palette.
#[derive(Debug, Clone, Deserialize)]
pub struct SubjectInput {
    pub name: String,
    pub color: Option<String>,
    pub icon: Option<String>,
}

/// Every subject that has not been deleted, in display order.
pub fn list(db: &Database) -> Result<Vec<SubjectDto>, CommandError> {
    let subjects = SubjectRepo::new(db).list()?;
    Ok(subjects.into_iter().map(SubjectDto::from).collect())
}

/// Creates a subject, filling in the colour and position the dialog does not
/// ask for.
pub fn create(db: &Database, input: SubjectInput) -> Result<SubjectDto, CommandError> {
    let repo = SubjectRepo::new(db);

    let name = normalize_name(&input.name)?;
    let icon = input
        .icon
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty());

    // Nothing chosen yet: hand out the next palette colour rather than leave
    // the subject grey. The count is of live subjects, so eight subjects keep
    // eight distinct colours even after some were deleted along the way —
    // unlike the position, which must never be reused.
    let color = match normalize_color(input.color.as_deref())? {
        Some(slug) => slug,
        None => palette_slug(repo.list()?.len()),
    };

    let position = repo.next_position()?;

    let subject = repo.create(&name, Some(&color), icon, position)?;
    Ok(subject.into())
}

/// Renames or recolours an existing subject. Its position is left alone —
/// editing a subject is not reordering the list.
pub fn update(db: &Database, id: String, input: SubjectInput) -> Result<SubjectDto, CommandError> {
    let repo = SubjectRepo::new(db);

    let name = normalize_name(&input.name)?;
    let color = normalize_color(input.color.as_deref())?;
    let icon = input
        .icon
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty());

    let current = repo
        .get(&id)?
        .filter(|subject| subject.deleted_at.is_none())
        .ok_or_else(|| CommandError::not_found("предмет"))?;

    repo.update(&id, &name, color.as_deref(), icon, current.position)?;

    Ok(SubjectDto {
        id,
        name,
        color,
        icon: icon.map(str::to_string),
        position: current.position,
    })
}

/// Soft-deletes a subject. Its sessions stay: statistics for a dropped
/// subject are still the student's history.
pub fn delete(db: &Database, id: &str) -> Result<(), CommandError> {
    let repo = SubjectRepo::new(db);

    repo.get(id)?
        .filter(|subject| subject.deleted_at.is_none())
        .ok_or_else(|| CommandError::not_found("предмет"))?;

    repo.soft_delete(id)?;
    Ok(())
}

// The `#[tauri::command]` wrappers. Each one only unwraps the managed
// `State` and calls the function above, so every rule they enforce stays
// reachable from a test with a plain in-memory `Database`.

#[tauri::command]
pub fn list_subjects(db: State<'_, Database>) -> Result<Vec<SubjectDto>, CommandError> {
    list(&db)
}

#[tauri::command]
pub fn create_subject(
    db: State<'_, Database>,
    input: SubjectInput,
) -> Result<SubjectDto, CommandError> {
    create(&db, input)
}

#[tauri::command]
pub fn update_subject(
    db: State<'_, Database>,
    id: String,
    input: SubjectInput,
) -> Result<SubjectDto, CommandError> {
    update(&db, id, input)
}

#[tauri::command]
pub fn delete_subject(db: State<'_, Database>, id: String) -> Result<(), CommandError> {
    delete(&db, &id)
}
