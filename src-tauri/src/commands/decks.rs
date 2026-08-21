//! Decks: creating, renaming, and listing them with their card counts.

use serde::{Deserialize, Serialize};
use tauri::State;

use crate::core::deck::{normalize_deck_name, normalize_description};
use crate::db::decks::DeckRepo;
use crate::db::subjects::SubjectRepo;
use crate::db::Database;

use super::CommandError;

/// A deck as the cards screen draws it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DeckView {
    pub id: String,
    pub subject_id: Option<String>,
    pub name: String,
    pub description: Option<String>,
    /// Live cards in the deck. The screen shows it next to the name, so it
    /// comes with the list rather than as a query per deck.
    pub card_count: i64,
}

/// What the deck dialog collects.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct DeckInput {
    pub subject_id: Option<String>,
    pub name: String,
    pub description: Option<String>,
}

/// Checks that a subject exists and is not deleted, so a deck cannot be
/// filed under something that is gone.
fn check_subject(db: &Database, subject_id: Option<&str>) -> Result<(), CommandError> {
    let Some(id) = subject_id else {
        return Ok(());
    };

    match SubjectRepo::new(db).get(id)? {
        Some(subject) if subject.deleted_at.is_none() => Ok(()),
        _ => Err(CommandError::not_found("предмет")),
    }
}

pub fn list(db: &Database) -> Result<Vec<DeckView>, CommandError> {
    let repo = DeckRepo::new(db);
    let counts = repo.card_counts()?;

    Ok(repo
        .list()?
        .into_iter()
        .map(|deck| {
            let card_count = counts
                .iter()
                .find(|(id, _)| id == &deck.id)
                .map(|(_, count)| *count)
                .unwrap_or(0);

            DeckView {
                id: deck.id,
                subject_id: deck.subject_id,
                name: deck.name,
                description: deck.description,
                card_count,
            }
        })
        .collect())
}

pub fn create(db: &Database, input: DeckInput) -> Result<DeckView, CommandError> {
    let name = normalize_deck_name(&input.name)?;
    let description = normalize_description(input.description.as_deref());
    check_subject(db, input.subject_id.as_deref())?;

    let deck =
        DeckRepo::new(db).create(input.subject_id.as_deref(), &name, description.as_deref())?;

    Ok(DeckView {
        id: deck.id,
        subject_id: deck.subject_id,
        name: deck.name,
        description: deck.description,
        card_count: 0,
    })
}

pub fn update(db: &Database, id: &str, input: DeckInput) -> Result<DeckView, CommandError> {
    let name = normalize_deck_name(&input.name)?;
    let description = normalize_description(input.description.as_deref());
    check_subject(db, input.subject_id.as_deref())?;

    let repo = DeckRepo::new(db);
    let existing = repo
        .get(id)?
        .filter(|deck| deck.deleted_at.is_none())
        .ok_or_else(|| CommandError::not_found("колода"))?;

    repo.update(
        &existing.id,
        input.subject_id.as_deref(),
        &name,
        description.as_deref(),
    )?;

    list(db)?
        .into_iter()
        .find(|deck| deck.id == id)
        .ok_or_else(|| CommandError::not_found("колода"))
}

/// Soft-deletes a deck. Its cards stay: the reviews they carry are part of
/// the statistics, and a deck brought back should come back whole.
pub fn delete(db: &Database, id: &str) -> Result<(), CommandError> {
    let repo = DeckRepo::new(db);

    repo.get(id)?
        .filter(|deck| deck.deleted_at.is_none())
        .ok_or_else(|| CommandError::not_found("колода"))?;
    repo.soft_delete(id)?;

    Ok(())
}

#[tauri::command]
pub fn list_decks(db: State<'_, Database>) -> Result<Vec<DeckView>, CommandError> {
    list(&db)
}

#[tauri::command]
pub fn create_deck(db: State<'_, Database>, input: DeckInput) -> Result<DeckView, CommandError> {
    create(&db, input)
}

#[tauri::command]
pub fn update_deck(
    db: State<'_, Database>,
    id: String,
    input: DeckInput,
) -> Result<DeckView, CommandError> {
    update(&db, &id, input)
}

#[tauri::command]
pub fn delete_deck(db: State<'_, Database>, id: String) -> Result<(), CommandError> {
    delete(&db, &id)
}
