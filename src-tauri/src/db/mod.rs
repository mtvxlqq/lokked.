//! SQLite persistence layer.
//!
//! This module owns the connection/pool, opens the database under the app's
//! data directory, and applies [`migrations`] on startup. Everything above it
//! (commands, [`crate::core`]) works with plain Rust types, never with rows.
//!
//! Deliberately empty for now: the driver choice (`rusqlite` with a bundled
//! SQLite vs `sqlx`) materially affects the Android build, so it is made in
//! the step that actually implements persistence rather than in the skeleton.

pub mod migrations;

// TODO: `pub struct Database` wrapping the connection/pool.
// TODO: `pub fn open(app: &tauri::AppHandle) -> Result<Database, DbError>` —
//       resolve the app data dir, open the file, run migrations.
// TODO: `pub enum DbError` covering open/migrate/query failures.
