//! Thin Tauri command layer.
//!
//! Commands in this module stay deliberately thin: they parse arguments,
//! delegate to [`crate::core`] / [`crate::db`] / [`crate::platform`], and map
//! the result into something serde can hand to the frontend. No domain logic
//! lives here, so it stays testable without a running Tauri app.
//!
//! Split by screen concern: [`subjects`], [`presets`], [`session`],
//! [`today`], [`settings`], [`decks`], [`cards`], [`import`], [`study`],
//! [`stats`] and [`streak`].
//! Anything shared — the error type the frontend sees — lives here.

use serde::Serialize;

use crate::core::card::CardError;
use crate::core::deck::DeckError;
use crate::core::import::ImportError;
use crate::core::preset::PresetError;
use crate::core::review::UnknownGrade;
use crate::core::scheduler::UnknownMode;
use crate::core::settings::SettingsError;
use crate::core::stats::time::UnknownRange;
use crate::core::subject::SubjectError;
use crate::db::DbError;

pub mod cards;
pub mod decks;
pub mod import;
pub mod presets;
pub mod session;
pub mod settings;
pub mod stats;
pub mod streak;
pub mod study;
pub mod subjects;
pub mod today;

/// A failed command, as the frontend sees it.
///
/// `kind` is what the UI branches on; `message` is already user-facing
/// Russian, because the validation errors in [`crate::core`] carry the only
/// wording that knows which field went wrong. A `Database` message is the
/// exception — it is a technical string, shown as a fallback rather than as
/// something a student can act on.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CommandError {
    pub kind: ErrorKind,
    pub message: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ErrorKind {
    /// The input was rejected by a `core` validator. The user can fix it.
    Validation,
    /// A row the command needed is gone — deleted in another window, or a
    /// stale id from a screen that was not refreshed.
    NotFound,
    /// SQLite failed. Not the user's fault and not the user's problem.
    Database,
    /// The action does not exist from the current state — starting a second
    /// session, or pausing one that is already paused.
    Conflict,
}

impl CommandError {
    pub fn not_found(what: &str) -> Self {
        Self {
            kind: ErrorKind::NotFound,
            message: format!("{what} не найден"),
        }
    }
}

impl From<DbError> for CommandError {
    fn from(err: DbError) -> Self {
        Self {
            kind: ErrorKind::Database,
            message: err.to_string(),
        }
    }
}

impl From<SubjectError> for CommandError {
    fn from(err: SubjectError) -> Self {
        Self {
            kind: ErrorKind::Validation,
            message: err.to_string(),
        }
    }
}

impl From<PresetError> for CommandError {
    fn from(err: PresetError) -> Self {
        Self {
            kind: ErrorKind::Validation,
            message: err.to_string(),
        }
    }
}

impl From<SettingsError> for CommandError {
    fn from(err: SettingsError) -> Self {
        Self {
            kind: ErrorKind::Validation,
            message: err.to_string(),
        }
    }
}

impl From<CardError> for CommandError {
    fn from(err: CardError) -> Self {
        Self {
            kind: ErrorKind::Validation,
            message: err.to_string(),
        }
    }
}

impl From<DeckError> for CommandError {
    fn from(err: DeckError) -> Self {
        Self {
            kind: ErrorKind::Validation,
            message: err.to_string(),
        }
    }
}

impl From<UnknownGrade> for CommandError {
    fn from(err: UnknownGrade) -> Self {
        Self {
            kind: ErrorKind::Validation,
            message: err.to_string(),
        }
    }
}

impl From<UnknownMode> for CommandError {
    fn from(err: UnknownMode) -> Self {
        Self {
            kind: ErrorKind::Validation,
            message: err.to_string(),
        }
    }
}

impl From<UnknownRange> for CommandError {
    fn from(err: UnknownRange) -> Self {
        Self {
            kind: ErrorKind::Validation,
            message: err.to_string(),
        }
    }
}

impl From<ImportError> for CommandError {
    fn from(err: ImportError) -> Self {
        Self {
            kind: ErrorKind::Validation,
            message: err.to_string(),
        }
    }
}

/// Health check for the Rust ↔ TypeScript bridge.
///
/// The frontend calls this on startup; seeing `"pong"` in the window proves
/// the IPC layer is wired up correctly.
#[tauri::command]
pub fn ping() -> String {
    "pong".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ping_returns_pong() {
        assert_eq!(ping(), "pong");
    }
}
