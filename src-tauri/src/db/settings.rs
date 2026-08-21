//! The `settings` table: one row per setting, key to string.
//!
//! Deliberately untyped at this level — what a value means is
//! [`crate::core::settings`]'s business. The table has no `deleted_at`: a
//! setting is either stored or absent, and «absent» already means «use the
//! default», so there is nothing a soft delete would express.

use rusqlite::{params, OptionalExtension};

use super::{Database, DbError};

/// Reads and writes single settings.
pub struct SettingsRepo<'a> {
    db: &'a Database,
}

impl<'a> SettingsRepo<'a> {
    pub fn new(db: &'a Database) -> Self {
        Self { db }
    }

    /// The stored value, or `None` if the setting has never been written.
    pub fn get(&self, key: &str) -> Result<Option<String>, DbError> {
        self.db
            .connection()
            .query_row(
                "SELECT value FROM settings WHERE key = ?1",
                params![key],
                |row| row.get(0),
            )
            .optional()
            .map_err(DbError::from)
    }

    /// Every stored setting, as `(key, value)`.
    pub fn all(&self) -> Result<Vec<(String, String)>, DbError> {
        let conn = self.db.connection();
        let mut stmt = conn.prepare("SELECT key, value FROM settings ORDER BY key")?;
        let rows = stmt.query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?;
        rows.collect::<rusqlite::Result<Vec<_>>>()
            .map_err(DbError::from)
    }

    /// Writes a setting, replacing whatever was there.
    pub fn set(&self, key: &str, value: &str) -> Result<(), DbError> {
        self.db.connection().execute(
            "INSERT INTO settings (key, value, updated_at)
             VALUES (?1, ?2, ?3)
             ON CONFLICT(key) DO UPDATE SET value = ?2, updated_at = ?3",
            params![key, value, chrono::Utc::now()],
        )?;
        Ok(())
    }
}
