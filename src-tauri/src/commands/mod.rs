//! Thin Tauri command layer.
//!
//! Commands in this module stay deliberately thin: they parse arguments,
//! delegate to [`crate::core`] / [`crate::db`] / [`crate::platform`], and map
//! the result into something serde can hand to the frontend. No domain logic
//! lives here, so it stays testable without a running Tauri app.
//!
//! Split by screen concern: [`subjects`], [`presets`], [`session`],
//! [`settings`] and [`today`].
//! Anything shared — the error type the frontend sees — lives here.

use serde::Serialize;

use crate::core::preset::PresetError;
use crate::core::settings::SettingsError;
use crate::core::subject::SubjectError;
use crate::db::DbError;

pub mod presets;
pub mod session;
pub mod settings;
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
