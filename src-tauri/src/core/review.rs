//! What an answer to a card is.
//!
//! Four grades, as on the review screen. They are stored as slugs in
//! `reviews.result`, and `correct` is derived from them rather than asked
//! for separately — one source of truth for «вспомнил или нет».

use std::fmt;

use serde::{Deserialize, Serialize};

/// How well a card was recalled.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Grade {
    /// Не помню.
    Again,
    /// Вспомнил с трудом.
    Hard,
    /// Знаю.
    Good,
    /// Легко.
    Easy,
}

/// An answer the review screen sent that this module does not know.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnknownGrade(pub String);

impl fmt::Display for UnknownGrade {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "неизвестная оценка: {}", self.0)
    }
}

impl std::error::Error for UnknownGrade {}

impl Grade {
    /// The slug stored in `reviews.result`.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Again => "again",
            Self::Hard => "hard",
            Self::Good => "good",
            Self::Easy => "easy",
        }
    }

    pub fn parse(raw: &str) -> Result<Self, UnknownGrade> {
        match raw {
            "again" => Ok(Self::Again),
            "hard" => Ok(Self::Hard),
            "good" => Ok(Self::Good),
            "easy" => Ok(Self::Easy),
            other => Err(UnknownGrade(other.to_string())),
        }
    }

    /// Whether the card counts as recalled.
    ///
    /// «С трудом» counts: the card will come round sooner, but the student
    /// did produce the answer, and calling that a mistake in the summary
    /// would misrepresent the run.
    pub fn is_correct(self) -> bool {
        !matches!(self, Self::Again)
    }
}
