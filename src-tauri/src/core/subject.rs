//! Validation rules for a subject.
//!
//! The UI collects a name and a colour; this module decides whether that pair
//! is storable and hands back the normalised form the repository writes. It is
//! the only place those rules live — the frontend may grey out a disabled
//! button early, but it never decides on its own what counts as valid.

use std::fmt;

/// How many colours the subject palette has. Slugs run `subject-1` through
/// `subject-8`, matching `--color-subject-N` in `src/styles/tokens.css`.
pub const PALETTE_SIZE: usize = 8;

/// Longest accepted subject name, in characters (not bytes — «Математический
/// анализ» is 21 characters and 40 bytes).
pub const MAX_NAME_LEN: usize = 60;

/// Why a subject was rejected.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SubjectError {
    /// The name was blank, or nothing but whitespace.
    EmptyName,
    /// The name exceeded [`MAX_NAME_LEN`] characters.
    NameTooLong { max: usize },
    /// The colour was not one of the palette slugs.
    UnknownColor(String),
}

impl fmt::Display for SubjectError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyName => write!(f, "название предмета не может быть пустым"),
            Self::NameTooLong { max } => {
                write!(f, "название предмета длиннее {max} символов")
            }
            Self::UnknownColor(slug) => write!(f, "неизвестный цвет палитры: {slug}"),
        }
    }
}

impl std::error::Error for SubjectError {}

/// Trims surrounding whitespace and checks the length.
///
/// Interior whitespace is left alone: «Теория вероятностей и статистика» is a
/// legitimate name, and silently rewriting what someone typed is worse than
/// storing it verbatim.
pub fn normalize_name(raw: &str) -> Result<String, SubjectError> {
    let trimmed = raw.trim();

    if trimmed.is_empty() {
        return Err(SubjectError::EmptyName);
    }
    if trimmed.chars().count() > MAX_NAME_LEN {
        return Err(SubjectError::NameTooLong { max: MAX_NAME_LEN });
    }

    Ok(trimmed.to_string())
}

/// Checks that a colour is a palette slug.
///
/// Colours are stored as `subject-N`, never as hex: the palette lives in the
/// design tokens, and a stored `#7E9CC4` would silently drift the day the
/// palette is retuned.
pub fn normalize_color(raw: Option<&str>) -> Result<Option<String>, SubjectError> {
    let Some(slug) = raw.map(str::trim).filter(|s| !s.is_empty()) else {
        return Ok(None);
    };

    let index = slug
        .strip_prefix("subject-")
        .and_then(|n| n.parse::<usize>().ok())
        .filter(|n| (1..=PALETTE_SIZE).contains(n));

    match index {
        Some(_) => Ok(Some(slug.to_string())),
        None => Err(SubjectError::UnknownColor(slug.to_string())),
    }
}

/// The palette slug at `index`, wrapping around. Used to give a new subject a
/// colour without asking: the Nth subject gets the Nth colour.
pub fn palette_slug(index: usize) -> String {
    format!("subject-{}", index % PALETTE_SIZE + 1)
}
