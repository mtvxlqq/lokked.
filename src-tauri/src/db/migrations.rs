//! Schema migrations.
//!
//! The migrations themselves are SQL files under `src-tauri/migrations/`, named
//! with their version prefix. This module only loads and applies them.
//!
//! Migrations are append-only and forward-only: each one gets the next integer
//! version and is **never** edited once it has shipped — a schema change means
//! a new file. The applied version is tracked with SQLite's `PRAGMA
//! user_version`.
//!
//! TODO: `pub fn all() -> &'static [Migration]` — the ordered list, embedded at
//!       compile time with `include_str!` so a release binary needs no files
//!       on disk.
//! TODO: `pub fn apply(conn: &mut Connection) -> Result<(), DbError>` running
//!       every migration newer than `user_version` inside one transaction.
//! TODO: migration 1 — subjects, sessions, decks, cards, reviews. Per CLAUDE.md:
//!       UUIDv7 TEXT primary keys, `created_at` / `updated_at` / `deleted_at` on
//!       every table, `sessions` and `reviews` append-only.
