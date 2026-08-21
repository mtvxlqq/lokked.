//! Bulk import and export.
//!
//! The screen works in two steps, and so does this module: first a preview,
//! which parses and writes nothing, then the import itself, which validates
//! every card and writes them in one transaction. Nothing is stored between
//! the two — the frontend holds the preview it was shown and hands the cards
//! back, so what is written is exactly what was on screen.

use serde::Serialize;
use tauri::State;

use crate::core::import::{
    parse_lecture_json, parse_text, to_text, ImportOptions, ImportPreview, ParsedCard,
};
use crate::db::cards::{CardRepo, NewCard};
use crate::db::Database;

use super::cards::{clean, CardInput};
use super::CommandError;

/// Which format the pasted text turned out to be in.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ImportFormat {
    /// Cards divided by separators.
    Text,
    /// A prepared JSON file of lecture cards.
    LectureJson,
}

/// A preview plus the format it was read as.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ImportReport {
    pub format: ImportFormat,
    #[serde(flatten)]
    pub preview: ImportPreview,
}

/// Parses whatever arrived, in whichever of the two formats it is in.
///
/// The format is decided by the content rather than by asking: a file that
/// starts with `{` is the JSON one, and everything else is the text format.
/// Separators only matter for the latter.
pub fn preview(raw: &str, options: &ImportOptions) -> Result<ImportReport, CommandError> {
    if raw.trim_start().starts_with('{') {
        let import = parse_lecture_json(raw)?;

        return Ok(ImportReport {
            format: ImportFormat::LectureJson,
            preview: import.preview,
        });
    }

    Ok(ImportReport {
        format: ImportFormat::Text,
        preview: parse_text(raw, options),
    })
}

/// Writes the previewed cards into a deck.
///
/// Every card goes through the same validation as one typed by hand, so an
/// import cannot smuggle in a blank side or a tag with a comma. The write is
/// one transaction: a deck is either fully imported or untouched.
pub fn commit(db: &Database, deck_id: &str, cards: &[ParsedCard]) -> Result<usize, CommandError> {
    super::cards::check_deck(db, deck_id)?;

    let cleaned = cards
        .iter()
        .map(|card| {
            clean(&CardInput {
                front: card.front.clone(),
                back: card.back.clone(),
                hint: card.hint.clone(),
                tags: card.tags.clone(),
            })
        })
        .collect::<Result<Vec<_>, _>>()?;

    let rows: Vec<NewCard<'_>> = cleaned
        .iter()
        .map(|card| NewCard {
            deck_id,
            front: &card.front,
            back: &card.back,
            hint: card.hint.as_deref(),
            tags: card.tags.as_deref(),
        })
        .collect();

    Ok(CardRepo::new(db).create_many(&rows)?)
}

/// A deck written back out in the text import format.
pub fn export(
    db: &Database,
    deck_id: &str,
    options: &ImportOptions,
) -> Result<String, CommandError> {
    let cards: Vec<ParsedCard> = super::cards::list(db, deck_id)?
        .into_iter()
        .map(|card| ParsedCard {
            front: card.front,
            back: card.back,
            hint: card.hint,
            tags: card.tags,
        })
        .collect();

    Ok(to_text(&cards, options))
}

/// Builds the options from what the import screen has in its two fields,
/// falling back to the defaults when they are left alone.
fn options(
    card_separator: Option<String>,
    side_separator: Option<String>,
) -> Result<ImportOptions, CommandError> {
    match (card_separator, side_separator) {
        (Some(card), Some(side)) => Ok(ImportOptions::new(&card, &side)?),
        _ => Ok(ImportOptions::default()),
    }
}

#[tauri::command]
pub fn preview_import(
    text: String,
    card_separator: Option<String>,
    side_separator: Option<String>,
) -> Result<ImportReport, CommandError> {
    preview(&text, &options(card_separator, side_separator)?)
}

#[tauri::command]
pub fn import_cards(
    db: State<'_, Database>,
    deck_id: String,
    cards: Vec<ParsedCard>,
) -> Result<usize, CommandError> {
    commit(&db, &deck_id, &cards)
}

#[tauri::command]
pub fn export_deck(
    db: State<'_, Database>,
    deck_id: String,
    card_separator: Option<String>,
    side_separator: Option<String>,
) -> Result<String, CommandError> {
    export(&db, &deck_id, &options(card_separator, side_separator)?)
}
