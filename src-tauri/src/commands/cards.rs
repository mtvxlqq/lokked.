//! Cards: the editor's create, edit, move and delete.
//!
//! Search and tag filtering are not here. A deck is a few hundred cards, the
//! screen already has all of them, and filtering in the frontend is instant
//! where a round trip per keystroke would not be.

use serde::{Deserialize, Serialize};
use tauri::State;

use crate::core::card::{join_tags, normalize_hint, normalize_side, normalize_tags, split_tags};
use crate::db::cards::{CardRepo, NewCard};
use crate::db::decks::DeckRepo;
use crate::db::Database;

use super::CommandError;

/// A card as the editor draws it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CardView {
    pub id: String,
    pub deck_id: String,
    pub front: String,
    pub back: String,
    pub hint: Option<String>,
    /// Split out of the stored column, so the frontend never parses it.
    pub tags: Vec<String>,
}

/// What the card dialog collects.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct CardInput {
    pub front: String,
    pub back: String,
    pub hint: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
}

/// A validated card, ready for the repository.
pub struct CleanCard {
    pub front: String,
    pub back: String,
    pub hint: Option<String>,
    pub tags: Option<String>,
}

/// Runs one card through [`crate::core::card`]'s rules.
pub fn clean(input: &CardInput) -> Result<CleanCard, CommandError> {
    Ok(CleanCard {
        front: normalize_side(&input.front)?,
        back: normalize_side(&input.back)?,
        hint: normalize_hint(input.hint.as_deref()),
        tags: join_tags(&normalize_tags(&input.tags)?),
    })
}

fn view(card: crate::db::cards::Card) -> CardView {
    CardView {
        tags: split_tags(card.tags.as_deref()),
        id: card.id,
        deck_id: card.deck_id,
        front: card.front,
        back: card.back,
        hint: card.hint,
    }
}

/// Checks that a deck exists and is not deleted.
pub fn check_deck(db: &Database, deck_id: &str) -> Result<(), CommandError> {
    match DeckRepo::new(db).get(deck_id)? {
        Some(deck) if deck.deleted_at.is_none() => Ok(()),
        _ => Err(CommandError::not_found("колода")),
    }
}

pub fn list(db: &Database, deck_id: &str) -> Result<Vec<CardView>, CommandError> {
    check_deck(db, deck_id)?;

    Ok(CardRepo::new(db)
        .list_for_deck(deck_id)?
        .into_iter()
        .map(view)
        .collect())
}

pub fn create(db: &Database, deck_id: &str, input: CardInput) -> Result<CardView, CommandError> {
    check_deck(db, deck_id)?;
    let clean = clean(&input)?;

    let card = CardRepo::new(db).create(NewCard {
        deck_id,
        front: &clean.front,
        back: &clean.back,
        hint: clean.hint.as_deref(),
        tags: clean.tags.as_deref(),
    })?;

    Ok(view(card))
}

pub fn update(db: &Database, id: &str, input: CardInput) -> Result<CardView, CommandError> {
    let clean = clean(&input)?;
    let repo = CardRepo::new(db);

    repo.get(id)?
        .filter(|card| card.deleted_at.is_none())
        .ok_or_else(|| CommandError::not_found("карточка"))?;
    repo.update(
        id,
        &clean.front,
        &clean.back,
        clean.hint.as_deref(),
        clean.tags.as_deref(),
    )?;

    repo.get(id)?
        .map(view)
        .ok_or_else(|| CommandError::not_found("карточка"))
}

pub fn move_to_deck(db: &Database, id: &str, deck_id: &str) -> Result<CardView, CommandError> {
    check_deck(db, deck_id)?;
    let repo = CardRepo::new(db);

    repo.get(id)?
        .filter(|card| card.deleted_at.is_none())
        .ok_or_else(|| CommandError::not_found("карточка"))?;
    repo.move_to_deck(id, deck_id)?;

    repo.get(id)?
        .map(view)
        .ok_or_else(|| CommandError::not_found("карточка"))
}

/// Soft-deletes a card. Its reviews stay — they are what the statistics are
/// made of, and they are append-only.
pub fn delete(db: &Database, id: &str) -> Result<(), CommandError> {
    let repo = CardRepo::new(db);

    repo.get(id)?
        .filter(|card| card.deleted_at.is_none())
        .ok_or_else(|| CommandError::not_found("карточка"))?;
    repo.soft_delete(id)?;

    Ok(())
}

#[tauri::command]
pub fn list_cards(db: State<'_, Database>, deck_id: String) -> Result<Vec<CardView>, CommandError> {
    list(&db, &deck_id)
}

#[tauri::command]
pub fn create_card(
    db: State<'_, Database>,
    deck_id: String,
    input: CardInput,
) -> Result<CardView, CommandError> {
    create(&db, &deck_id, input)
}

#[tauri::command]
pub fn update_card(
    db: State<'_, Database>,
    id: String,
    input: CardInput,
) -> Result<CardView, CommandError> {
    update(&db, &id, input)
}

#[tauri::command]
pub fn move_card(
    db: State<'_, Database>,
    id: String,
    deck_id: String,
) -> Result<CardView, CommandError> {
    move_to_deck(&db, &id, &deck_id)
}

#[tauri::command]
pub fn delete_card(db: State<'_, Database>, id: String) -> Result<(), CommandError> {
    delete(&db, &id)
}
