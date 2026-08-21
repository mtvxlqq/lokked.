//! Validation rules for a card: its two sides, its hint and its tags.
//!
//! The text itself is left exactly as it was typed apart from the whitespace
//! around it — a card's back is a paragraph of Markdown with LaTeX in it, and
//! rewriting any of that here would corrupt formulas. What this module does
//! decide is what may not be stored at all: a blank side, a tag with a comma
//! in it (commas separate tags in the stored column), and lists of tags long
//! enough to be a mistake.

use std::fmt;

/// Longest accepted tag, in characters.
///
/// Generous on purpose: a tag is often a phrase rather than a word — the
/// topic of a lecture («Кривые и области в $\mathbb{R}^m$. Предел функции
/// многих переменных») is exactly the kind of thing worth filtering a deck
/// by, and it is 67 characters long.
pub const MAX_TAG_LEN: usize = 80;

/// Most tags one card may carry. Past this it is not a tag list any more.
pub const MAX_TAGS: usize = 12;

/// Why a card was rejected.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CardError {
    /// The front or the back was blank.
    EmptySide,
    TagWithComma(String),
    TagTooLong {
        max: usize,
    },
    TooManyTags {
        max: usize,
    },
}

impl fmt::Display for CardError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptySide => write!(f, "у карточки должны быть заполнены обе стороны"),
            Self::TagWithComma(tag) => {
                write!(f, "в теге не может быть запятой: {tag}")
            }
            Self::TagTooLong { max } => write!(f, "тег длиннее {max} символов"),
            Self::TooManyTags { max } => write!(f, "у карточки больше {max} тегов"),
        }
    }
}

impl std::error::Error for CardError {}

/// Trims the whitespace around a side and refuses a blank one.
pub fn normalize_side(raw: &str) -> Result<String, CardError> {
    let trimmed = raw.trim();

    if trimmed.is_empty() {
        return Err(CardError::EmptySide);
    }

    Ok(trimmed.to_string())
}

/// A hint, or nothing at all — a blank hint is stored as `NULL`, not as `""`.
pub fn normalize_hint(raw: Option<&str>) -> Option<String> {
    raw.map(str::trim)
        .filter(|hint| !hint.is_empty())
        .map(str::to_string)
}

/// Trims tags, drops blanks, removes repeats and checks the limits.
///
/// Repeats are compared case-insensitively but the first spelling is what is
/// kept: the student sees the tag as they wrote it.
pub fn normalize_tags(raw: &[String]) -> Result<Vec<String>, CardError> {
    let mut tags: Vec<String> = Vec::new();

    for tag in raw {
        let tag = tag.trim();
        if tag.is_empty() {
            continue;
        }
        if tag.contains(',') {
            return Err(CardError::TagWithComma(tag.to_string()));
        }
        if tag.chars().count() > MAX_TAG_LEN {
            return Err(CardError::TagTooLong { max: MAX_TAG_LEN });
        }
        if tags
            .iter()
            .any(|kept| kept.to_lowercase() == tag.to_lowercase())
        {
            continue;
        }
        tags.push(tag.to_string());
    }

    if tags.len() > MAX_TAGS {
        return Err(CardError::TooManyTags { max: MAX_TAGS });
    }

    Ok(tags)
}

/// Tags as they are stored in `cards.tags`: comma-separated, or `NULL` when
/// there are none.
pub fn join_tags(tags: &[String]) -> Option<String> {
    (!tags.is_empty()).then(|| tags.join(","))
}

/// The inverse of [`join_tags`].
pub fn split_tags(stored: Option<&str>) -> Vec<String> {
    stored
        .unwrap_or_default()
        .split(',')
        .map(str::trim)
        .filter(|tag| !tag.is_empty())
        .map(str::to_string)
        .collect()
}
