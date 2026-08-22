//! The `cards` table: two sides, an optional hint, and tags.
//!
//! The scheduling columns (`ease`, `interval_days`, `due_at`, `reps`,
//! `lapses`) are a cache of what the adaptive picker computed, not state of
//! their own — see [`CardRepo::cache_weight`]. Editing a card leaves them
//! alone, and losing them costs nothing: `reviews` is the source of truth and
//! every weight can be recomputed from it.

use chrono::{DateTime, Utc};
use rusqlite::{params, OptionalExtension};
use uuid::Uuid;

use super::{Database, DbError};

/// A card as stored in `cards`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Card {
    pub id: String,
    pub deck_id: String,
    pub front: String,
    pub back: String,
    pub hint: Option<String>,
    /// Comma-separated, as [`crate::core::card::join_tags`] writes them.
    pub tags: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

/// What the adaptive picker last computed for a card.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WeightCache {
    /// The weight itself, stored in `ease`.
    pub weight: f64,
    /// How many answers that weight was computed from.
    pub reps: i64,
    /// How many of those were «не помню».
    pub lapses: i64,
}

/// A card on its way into the table.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewCard<'a> {
    pub deck_id: &'a str,
    pub front: &'a str,
    pub back: &'a str,
    pub hint: Option<&'a str>,
    pub tags: Option<&'a str>,
}

fn row_to_card(row: &rusqlite::Row<'_>) -> rusqlite::Result<Card> {
    Ok(Card {
        id: row.get("id")?,
        deck_id: row.get("deck_id")?,
        front: row.get("front")?,
        back: row.get("back")?,
        hint: row.get("hint")?,
        tags: row.get("tags")?,
        created_at: row.get("created_at")?,
        updated_at: row.get("updated_at")?,
        deleted_at: row.get("deleted_at")?,
    })
}

const COLUMNS: &str = "id, deck_id, front, back, hint, tags, created_at, updated_at, deleted_at";

const INSERT: &str =
    "INSERT INTO cards (id, deck_id, front, back, hint, tags, created_at, updated_at, deleted_at)
     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?7, NULL)";

/// CRUD for `cards`.
pub struct CardRepo<'a> {
    db: &'a Database,
}

impl<'a> CardRepo<'a> {
    pub fn new(db: &'a Database) -> Self {
        Self { db }
    }

    pub fn create(&self, new: NewCard<'_>) -> Result<Card, DbError> {
        let id = Uuid::now_v7().to_string();

        self.db.connection().execute(
            INSERT,
            params![
                id,
                new.deck_id,
                new.front,
                new.back,
                new.hint,
                new.tags,
                Utc::now()
            ],
        )?;

        self.get(&id)?
            .ok_or(DbError::Query(rusqlite::Error::QueryReturnedNoRows))
    }

    /// Writes many cards at once, in one transaction.
    ///
    /// An import is all or nothing: a file that fails halfway through would
    /// otherwise leave a deck half full, with no way to tell which half.
    /// Returns how many rows were written.
    pub fn create_many(&self, cards: &[NewCard<'_>]) -> Result<usize, DbError> {
        let mut conn = self.db.connection();
        let tx = conn.transaction()?;

        {
            let mut stmt = tx.prepare(INSERT)?;
            for card in cards {
                stmt.execute(params![
                    Uuid::now_v7().to_string(),
                    card.deck_id,
                    card.front,
                    card.back,
                    card.hint,
                    card.tags,
                    Utc::now()
                ])?;
            }
        }

        tx.commit()?;
        Ok(cards.len())
    }

    /// Every live card of one deck, oldest first — import order, which for
    /// lecture cards is the order of the lectures themselves.
    pub fn list_for_deck(&self, deck_id: &str) -> Result<Vec<Card>, DbError> {
        let conn = self.db.connection();
        let mut stmt = conn.prepare(&format!(
            "SELECT {COLUMNS} FROM cards
             WHERE deck_id = ?1 AND deleted_at IS NULL
             ORDER BY created_at, id"
        ))?;
        let rows = stmt.query_map(params![deck_id], row_to_card)?;
        rows.collect::<rusqlite::Result<Vec<_>>>()
            .map_err(DbError::from)
    }

    /// A card by id, deleted or not.
    pub fn get(&self, id: &str) -> Result<Option<Card>, DbError> {
        self.db
            .connection()
            .query_row(
                &format!("SELECT {COLUMNS} FROM cards WHERE id = ?1"),
                params![id],
                row_to_card,
            )
            .optional()
            .map_err(DbError::from)
    }

    /// The cards with these ids, deleted ones included, in no particular
    /// order.
    ///
    /// One query instead of a `get` per id: the statistics screen looks up
    /// twenty cards at once, and twenty round trips through the connection
    /// mutex for a list that is already on screen would be silly.
    pub fn list_by_ids(&self, ids: &[String]) -> Result<Vec<Card>, DbError> {
        if ids.is_empty() {
            return Ok(Vec::new());
        }

        let placeholders = vec!["?"; ids.len()].join(", ");
        let conn = self.db.connection();
        let mut stmt = conn.prepare(&format!(
            "SELECT {COLUMNS} FROM cards WHERE id IN ({placeholders})"
        ))?;
        let rows = stmt.query_map(rusqlite::params_from_iter(ids), row_to_card)?;
        rows.collect::<rusqlite::Result<Vec<_>>>()
            .map_err(DbError::from)
    }

    pub fn update(
        &self,
        id: &str,
        front: &str,
        back: &str,
        hint: Option<&str>,
        tags: Option<&str>,
    ) -> Result<(), DbError> {
        self.db.connection().execute(
            "UPDATE cards SET front = ?2, back = ?3, hint = ?4, tags = ?5, updated_at = ?6
             WHERE id = ?1 AND deleted_at IS NULL",
            params![id, front, back, hint, tags, Utc::now()],
        )?;
        Ok(())
    }

    /// Moves a card to another deck, leaving everything else alone.
    pub fn move_to_deck(&self, id: &str, deck_id: &str) -> Result<(), DbError> {
        self.db.connection().execute(
            "UPDATE cards SET deck_id = ?2, updated_at = ?3
             WHERE id = ?1 AND deleted_at IS NULL",
            params![id, deck_id, Utc::now()],
        )?;
        Ok(())
    }

    /// Writes down what the picker last computed for a card.
    ///
    /// `ease` holds the weight, `reps` the number of answers behind it and
    /// `lapses` how many of those were «не помню». `interval_days` and
    /// `due_at` stay `NULL` on purpose: this scheduler has no due dates —
    /// every card stays in rotation and only its weight changes.
    ///
    /// `updated_at` is deliberately not touched. The cache is derived
    /// locally and recomputed at will; bumping the column would make every
    /// answered card look edited to the sync that column exists for.
    pub fn cache_weight(&self, id: &str, cache: WeightCache) -> Result<(), DbError> {
        self.db.connection().execute(
            "UPDATE cards SET ease = ?2, reps = ?3, lapses = ?4
             WHERE id = ?1 AND deleted_at IS NULL",
            params![id, cache.weight, cache.reps, cache.lapses],
        )?;
        Ok(())
    }

    /// The cached weight of a card, or `None` if it has never been answered.
    pub fn weight_cache(&self, id: &str) -> Result<Option<WeightCache>, DbError> {
        self.db
            .connection()
            .query_row(
                "SELECT ease, reps, lapses FROM cards WHERE id = ?1",
                params![id],
                |row| {
                    let weight: Option<f64> = row.get(0)?;
                    let reps: Option<i64> = row.get(1)?;
                    let lapses: Option<i64> = row.get(2)?;

                    Ok(weight.map(|weight| WeightCache {
                        weight,
                        reps: reps.unwrap_or(0),
                        lapses: lapses.unwrap_or(0),
                    }))
                },
            )
            .optional()
            .map(Option::flatten)
            .map_err(DbError::from)
    }

    pub fn soft_delete(&self, id: &str) -> Result<(), DbError> {
        let now = Utc::now();
        self.db.connection().execute(
            "UPDATE cards SET deleted_at = ?2, updated_at = ?2 WHERE id = ?1 AND deleted_at IS NULL",
            params![id, now],
        )?;
        Ok(())
    }
}
