//! Validation rules for a deck: the box a subject's cards live in.

use std::fmt;

/// Longest accepted deck name, in characters.
pub const MAX_NAME_LEN: usize = 80;

/// Why a deck was rejected.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DeckError {
    EmptyName,
    NameTooLong { max: usize },
}

impl fmt::Display for DeckError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyName => write!(f, "название колоды не может быть пустым"),
            Self::NameTooLong { max } => write!(f, "название колоды длиннее {max} символов"),
        }
    }
}

impl std::error::Error for DeckError {}

/// Trims the name and checks its length.
///
/// A deck name is longer than a subject's on purpose: it often carries the
/// range it covers — «Матанализ, § 25 — § 40» — and cutting that off would
/// make two decks indistinguishable in a list.
pub fn normalize_deck_name(raw: &str) -> Result<String, DeckError> {
    let trimmed = raw.trim();

    if trimmed.is_empty() {
        return Err(DeckError::EmptyName);
    }
    if trimmed.chars().count() > MAX_NAME_LEN {
        return Err(DeckError::NameTooLong { max: MAX_NAME_LEN });
    }

    Ok(trimmed.to_string())
}

/// The description, or nothing — a blank one is stored as `NULL`.
pub fn normalize_description(raw: Option<&str>) -> Option<String> {
    raw.map(str::trim)
        .filter(|text| !text.is_empty())
        .map(str::to_string)
}
