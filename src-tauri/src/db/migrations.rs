//! Schema migrations.
//!
//! The migrations themselves are SQL files under `src-tauri/migrations/`, named
//! with their version prefix. This module only loads and applies them.
//!
//! Migrations are append-only and forward-only: each one gets the next integer
//! version and is **never** edited once it has shipped — a schema change means
//! a new file. The applied version is tracked with SQLite's `PRAGMA
//! user_version`.

use rusqlite::Connection;

use super::DbError;

/// One schema migration: a version number and the SQL that gets a database
/// from `version - 1` to `version`.
pub struct Migration {
    pub version: i32,
    pub name: &'static str,
    pub sql: &'static str,
}

/// Every migration Lokked ships, embedded at compile time so a release
/// binary needs no `.sql` files on disk. Ordered by version; `apply` relies
/// on that order.
pub fn all() -> &'static [Migration] {
    &[Migration {
        version: 1,
        name: "init",
        sql: include_str!("../../migrations/0001_init.sql"),
    }]
}

/// Brings `conn` up to the newest known schema version.
///
/// Runs every migration newer than the database's current `PRAGMA
/// user_version` inside one transaction, then advances `user_version` to the
/// last version applied. Calling this on an already-current database is a
/// no-op: nothing runs, no error.
pub fn apply(conn: &mut Connection) -> Result<(), DbError> {
    let current_version: i32 = conn
        .query_row("PRAGMA user_version", [], |row| row.get(0))
        .map_err(|err| DbError::Migration(err.to_string()))?;

    let pending: Vec<&Migration> = all()
        .iter()
        .filter(|migration| migration.version > current_version)
        .collect();

    if pending.is_empty() {
        return Ok(());
    }

    let tx = conn
        .transaction()
        .map_err(|err| DbError::Migration(err.to_string()))?;

    let mut latest_version = current_version;
    for migration in pending {
        tx.execute_batch(migration.sql).map_err(|err| {
            DbError::Migration(format!(
                "migration {} ({}) failed: {err}",
                migration.version, migration.name
            ))
        })?;
        latest_version = migration.version;
    }

    // PRAGMA does not accept bound parameters; `latest_version` is our own
    // integer, never user input, so string interpolation is safe here.
    tx.execute_batch(&format!("PRAGMA user_version = {latest_version};"))
        .map_err(|err| DbError::Migration(err.to_string()))?;

    tx.commit()
        .map_err(|err| DbError::Migration(err.to_string()))
}
