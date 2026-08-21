//! SQLite persistence layer.
//!
//! This module owns the connection, opens the database under the app's data
//! directory, and applies [`migrations`] on startup. Everything above it
//! (commands, [`crate::core`]) works with plain Rust types, never with rows —
//! [`subjects`], [`presets`], [`sessions`], [`decks`], [`cards`],
//! [`reviews`] and [`settings`] each expose a repository with a typed CRUD
//! surface instead of raw SQL.
//!
//! A single [`rusqlite::Connection`] behind one [`std::sync::Mutex`] backs
//! the whole app rather than a pool: Lokked is a single-user app whose
//! Tauri commands already run sequentially on a thread pool, so there is no
//! concurrent-writer problem to solve, and a pool would be unused
//! complexity. `PRAGMA journal_mode = WAL` is still set — it protects
//! against corruption on a crash and does not block an external reader
//! (`sqlite3 lokked.sqlite3` while the app is running).

use std::error::Error;
use std::fmt;
use std::path::Path;
use std::sync::Mutex;

use rusqlite::Connection;
use tauri::Manager;

pub mod cards;
pub mod decks;
pub mod migrations;
pub mod presets;
pub mod reviews;
pub mod sessions;
pub mod settings;
pub mod subjects;

/// Something went wrong opening the database, migrating its schema, or
/// running a query.
#[derive(Debug)]
pub enum DbError {
    /// Could not open or create the database file.
    Open(String),
    /// A migration failed to apply.
    Migration(String),
    /// A query against an already-open, already-migrated database failed.
    Query(rusqlite::Error),
}

impl fmt::Display for DbError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Open(msg) => write!(f, "could not open database: {msg}"),
            Self::Migration(msg) => write!(f, "migration failed: {msg}"),
            Self::Query(err) => write!(f, "database query failed: {err}"),
        }
    }
}

impl Error for DbError {}

impl From<rusqlite::Error> for DbError {
    fn from(err: rusqlite::Error) -> Self {
        Self::Query(err)
    }
}

/// A handle to Lokked's SQLite database, already migrated to the current
/// schema.
pub struct Database {
    conn: Mutex<Connection>,
}

impl Database {
    /// Opens the app's real database, at a path resolved through Tauri's
    /// path API so it lands in the correct per-OS data directory.
    pub fn open(app: &tauri::AppHandle) -> Result<Database, DbError> {
        let dir = app
            .path()
            .app_data_dir()
            .map_err(|err| DbError::Open(err.to_string()))?;
        std::fs::create_dir_all(&dir).map_err(|err| DbError::Open(err.to_string()))?;

        Self::open_at(dir.join("lokked.sqlite3"))
    }

    /// Opens (creating if needed) a file-backed database at an explicit
    /// path, migrated to the current schema. Used by [`open`](Self::open)
    /// and directly by tests that want a real file on disk.
    pub fn open_at(path: impl AsRef<Path>) -> Result<Database, DbError> {
        let conn = Connection::open(path).map_err(|err| DbError::Open(err.to_string()))?;
        Self::from_connection(conn)
    }

    /// Opens an in-memory database, migrated to the current schema. Used by
    /// tests: fast, and leaves nothing on disk.
    pub fn open_in_memory() -> Result<Database, DbError> {
        let conn = Connection::open_in_memory().map_err(|err| DbError::Open(err.to_string()))?;
        Self::from_connection(conn)
    }

    fn from_connection(mut conn: Connection) -> Result<Database, DbError> {
        conn.execute_batch("PRAGMA foreign_keys = ON; PRAGMA journal_mode = WAL;")
            .map_err(|err| DbError::Open(err.to_string()))?;
        migrations::apply(&mut conn)?;

        Ok(Database {
            conn: Mutex::new(conn),
        })
    }

    /// Locks and hands out the underlying connection. Every repository
    /// method goes through this — there is no other way to reach the
    /// connection from outside this module.
    fn connection(&self) -> std::sync::MutexGuard<'_, Connection> {
        self.conn.lock().expect("database mutex poisoned")
    }
}
