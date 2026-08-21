//! The `decks` table: the box a subject's cards live in.

use chrono::{DateTime, Utc};
use rusqlite::{params, OptionalExtension};
use uuid::Uuid;

use super::{Database, DbError};

/// A deck of cards, as stored in `decks`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Deck {
    pub id: String,
    /// The subject this deck belongs to, or `None` for a deck of its own.
    pub subject_id: Option<String>,
    pub name: String,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

fn row_to_deck(row: &rusqlite::Row<'_>) -> rusqlite::Result<Deck> {
    Ok(Deck {
        id: row.get("id")?,
        subject_id: row.get("subject_id")?,
        name: row.get("name")?,
        description: row.get("description")?,
        created_at: row.get("created_at")?,
        updated_at: row.get("updated_at")?,
        deleted_at: row.get("deleted_at")?,
    })
}

const COLUMNS: &str = "id, subject_id, name, description, created_at, updated_at, deleted_at";

/// CRUD for `decks`. Deletion is soft, as everywhere: a deleted deck keeps
/// its cards and its review history, it simply stops being listed.
pub struct DeckRepo<'a> {
    db: &'a Database,
}

impl<'a> DeckRepo<'a> {
    pub fn new(db: &'a Database) -> Self {
        Self { db }
    }

    pub fn create(
        &self,
        subject_id: Option<&str>,
        name: &str,
        description: Option<&str>,
    ) -> Result<Deck, DbError> {
        let id = Uuid::now_v7().to_string();
        let now = Utc::now();

        self.db.connection().execute(
            "INSERT INTO decks (id, subject_id, name, description, created_at, updated_at, deleted_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?5, NULL)",
            params![id, subject_id, name, description, now],
        )?;

        self.get(&id)?
            .ok_or(DbError::Query(rusqlite::Error::QueryReturnedNoRows))
    }

    /// Every live deck, newest last — the order they were made in is the
    /// order a student thinks of them in.
    pub fn list(&self) -> Result<Vec<Deck>, DbError> {
        let conn = self.db.connection();
        let mut stmt = conn.prepare(&format!(
            "SELECT {COLUMNS} FROM decks WHERE deleted_at IS NULL ORDER BY created_at"
        ))?;
        let rows = stmt.query_map([], row_to_deck)?;
        rows.collect::<rusqlite::Result<Vec<_>>>()
            .map_err(DbError::from)
    }

    /// A deck by id, deleted or not — statistics still name a deleted deck.
    pub fn get(&self, id: &str) -> Result<Option<Deck>, DbError> {
        self.db
            .connection()
            .query_row(
                &format!("SELECT {COLUMNS} FROM decks WHERE id = ?1"),
                params![id],
                row_to_deck,
            )
            .optional()
            .map_err(DbError::from)
    }

    pub fn update(
        &self,
        id: &str,
        subject_id: Option<&str>,
        name: &str,
        description: Option<&str>,
    ) -> Result<(), DbError> {
        self.db.connection().execute(
            "UPDATE decks SET subject_id = ?2, name = ?3, description = ?4, updated_at = ?5
             WHERE id = ?1 AND deleted_at IS NULL",
            params![id, subject_id, name, description, Utc::now()],
        )?;
        Ok(())
    }

    pub fn soft_delete(&self, id: &str) -> Result<(), DbError> {
        let now = Utc::now();
        self.db.connection().execute(
            "UPDATE decks SET deleted_at = ?2, updated_at = ?2 WHERE id = ?1 AND deleted_at IS NULL",
            params![id, now],
        )?;
        Ok(())
    }

    /// How many live cards each live deck holds, as `(deck_id, count)`.
    ///
    /// One query rather than one per deck: the card list screen shows the
    /// number next to every deck at once.
    pub fn card_counts(&self) -> Result<Vec<(String, i64)>, DbError> {
        let conn = self.db.connection();
        let mut stmt = conn.prepare(
            "SELECT d.id, COUNT(c.id)
             FROM decks d
             LEFT JOIN cards c ON c.deck_id = d.id AND c.deleted_at IS NULL
             WHERE d.deleted_at IS NULL
             GROUP BY d.id",
        )?;
        let rows = stmt.query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?;
        rows.collect::<rusqlite::Result<Vec<_>>>()
            .map_err(DbError::from)
    }
}
