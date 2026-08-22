//! The `reviews` table: one row per answered card.
//!
//! Append-only, like `sessions`: a row is written when a card is graded and
//! is never edited or deleted afterwards. Everything the statistics screen
//! knows about how a card is going is derived from these rows, so losing or
//! rewriting one would rewrite history.

use chrono::{DateTime, Utc};
use rusqlite::params;
use uuid::Uuid;

use super::{Database, DbError};

/// An answer as stored in `reviews`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Review {
    pub id: String,
    pub card_id: String,
    pub reviewed_at: DateTime<Utc>,
    pub day_key: String,
    /// `'again' | 'hard' | 'good' | 'easy'`.
    pub result: String,
    pub correct: bool,
    /// `'classic' | 'blitz' | 'marathon' | 'weak'`.
    pub mode: String,
    /// Time to the answer being revealed, in milliseconds.
    pub think_ms: Option<i64>,
    /// Time to the grade being given, in milliseconds.
    pub total_ms: Option<i64>,
    pub device_id: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// An answer on its way into the table.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewReview<'a> {
    pub card_id: &'a str,
    pub reviewed_at: DateTime<Utc>,
    pub day_key: &'a str,
    pub result: &'a str,
    pub correct: bool,
    pub mode: &'a str,
    pub think_ms: Option<i64>,
    pub total_ms: Option<i64>,
    pub device_id: Option<&'a str>,
}

fn row_to_review(row: &rusqlite::Row<'_>) -> rusqlite::Result<Review> {
    Ok(Review {
        id: row.get("id")?,
        card_id: row.get("card_id")?,
        reviewed_at: row.get("reviewed_at")?,
        day_key: row.get("day_key")?,
        result: row.get("result")?,
        correct: row.get::<_, i64>("correct")? != 0,
        mode: row.get("mode")?,
        think_ms: row.get("think_ms")?,
        total_ms: row.get("total_ms")?,
        device_id: row.get("device_id")?,
        created_at: row.get("created_at")?,
    })
}

const COLUMNS: &str = "id, card_id, reviewed_at, day_key, result, correct, mode, think_ms,
                       total_ms, device_id, created_at";

/// Writes and reads `reviews`. There is deliberately no update and no delete.
pub struct ReviewRepo<'a> {
    db: &'a Database,
}

impl<'a> ReviewRepo<'a> {
    pub fn new(db: &'a Database) -> Self {
        Self { db }
    }

    pub fn create(&self, new: NewReview<'_>) -> Result<Review, DbError> {
        let id = Uuid::now_v7().to_string();

        self.db.connection().execute(
            "INSERT INTO reviews (id, card_id, reviewed_at, day_key, result, correct, mode,
                                  think_ms, total_ms, device_id, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            params![
                id,
                new.card_id,
                new.reviewed_at,
                new.day_key,
                new.result,
                new.correct as i64,
                new.mode,
                new.think_ms,
                new.total_ms,
                new.device_id,
                Utc::now(),
            ],
        )?;

        self.db
            .connection()
            .query_row(
                &format!("SELECT {COLUMNS} FROM reviews WHERE id = ?1"),
                params![id],
                row_to_review,
            )
            .map_err(DbError::from)
    }

    /// Every answer given to one card, oldest first.
    pub fn list_for_card(&self, card_id: &str) -> Result<Vec<Review>, DbError> {
        let conn = self.db.connection();
        let mut stmt = conn.prepare(&format!(
            "SELECT {COLUMNS} FROM reviews WHERE card_id = ?1 ORDER BY reviewed_at"
        ))?;
        let rows = stmt.query_map(params![card_id], row_to_review)?;
        rows.collect::<rusqlite::Result<Vec<_>>>()
            .map_err(DbError::from)
    }

    /// Every answer given on one study day, oldest first.
    pub fn list_for_day(&self, day_key: &str) -> Result<Vec<Review>, DbError> {
        let conn = self.db.connection();
        let mut stmt = conn.prepare(&format!(
            "SELECT {COLUMNS} FROM reviews WHERE day_key = ?1 ORDER BY reviewed_at"
        ))?;
        let rows = stmt.query_map(params![day_key], row_to_review)?;
        rows.collect::<rusqlite::Result<Vec<_>>>()
            .map_err(DbError::from)
    }

    /// Answers per study day over `[from_day, to_day]`.
    ///
    /// `(day_key, answered, correct)`, oldest day first, and only for days
    /// with answers — the caller fills the gaps, because a day with no
    /// answers has to be drawn differently from a day answered wrong.
    pub fn counts_by_day(
        &self,
        from_day: &str,
        to_day: &str,
    ) -> Result<Vec<(String, u32, u32)>, DbError> {
        let conn = self.db.connection();
        let mut stmt = conn.prepare(
            "SELECT day_key, COUNT(*), SUM(correct)
             FROM reviews
             WHERE day_key BETWEEN ?1 AND ?2
             GROUP BY day_key
             ORDER BY day_key",
        )?;
        let rows = stmt.query_map(params![from_day, to_day], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, i64>(1)? as u32,
                row.get::<_, i64>(2)? as u32,
            ))
        })?;
        rows.collect::<rusqlite::Result<Vec<_>>>()
            .map_err(DbError::from)
    }

    /// How every card has been going over `[from_day, to_day]`, across all
    /// decks.
    ///
    /// `(card_id, shown, correct)`. Deleted cards are left out: they are not
    /// worth going back to, whatever their accuracy was.
    pub fn accuracy_by_card_in_days(
        &self,
        from_day: &str,
        to_day: &str,
    ) -> Result<Vec<(String, u32, u32)>, DbError> {
        let conn = self.db.connection();
        let mut stmt = conn.prepare(
            "SELECT r.card_id, COUNT(*), SUM(r.correct)
             FROM reviews r
             JOIN cards c ON c.id = r.card_id
             WHERE c.deleted_at IS NULL AND r.day_key BETWEEN ?1 AND ?2
             GROUP BY r.card_id",
        )?;
        let rows = stmt.query_map(params![from_day, to_day], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, i64>(1)? as u32,
                row.get::<_, i64>(2)? as u32,
            ))
        })?;
        rows.collect::<rusqlite::Result<Vec<_>>>()
            .map_err(DbError::from)
    }

    /// The earliest study day an answer was recorded on, or `None` for a
    /// database nobody has answered anything in.
    pub fn earliest_day(&self) -> Result<Option<String>, DbError> {
        self.db
            .connection()
            .query_row("SELECT MIN(day_key) FROM reviews", [], |row| row.get(0))
            .map_err(DbError::from)
    }

    /// Every answer given in one deck since `since`, oldest first.
    ///
    /// `(card_id, reviewed_at, result)` — what the adaptive picker needs to
    /// weigh a card. The rows come back flat rather than grouped: grouping
    /// them into per-card histories is
    /// [`crate::core::scheduler::weights`]'s job, and it stays out of SQL so
    /// it can be tested without a database.
    pub fn history_for_deck(
        &self,
        deck_id: &str,
        since: DateTime<Utc>,
    ) -> Result<Vec<(String, DateTime<Utc>, String)>, DbError> {
        let conn = self.db.connection();
        let mut stmt = conn.prepare(
            "SELECT r.card_id, r.reviewed_at, r.result
             FROM reviews r
             JOIN cards c ON c.id = r.card_id
             WHERE c.deck_id = ?1 AND c.deleted_at IS NULL AND r.reviewed_at >= ?2
             ORDER BY r.reviewed_at",
        )?;
        let rows = stmt.query_map(params![deck_id, since], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?))
        })?;
        rows.collect::<rusqlite::Result<Vec<_>>>()
            .map_err(DbError::from)
    }

    /// How each card of one deck has been going since `since`.
    ///
    /// Returns `(card_id, shown, correct)` for every card that was answered
    /// at least once in the window; cards nobody has touched are simply
    /// absent, which is exactly what «слабые» needs — a card never seen has
    /// no accuracy to be bad.
    pub fn accuracy_by_card(
        &self,
        deck_id: &str,
        since: DateTime<Utc>,
    ) -> Result<Vec<(String, u32, u32)>, DbError> {
        let conn = self.db.connection();
        let mut stmt = conn.prepare(
            "SELECT r.card_id, COUNT(*), SUM(r.correct)
             FROM reviews r
             JOIN cards c ON c.id = r.card_id
             WHERE c.deck_id = ?1 AND c.deleted_at IS NULL AND r.reviewed_at >= ?2
             GROUP BY r.card_id",
        )?;
        let rows = stmt.query_map(params![deck_id, since], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, i64>(1)? as u32,
                row.get::<_, i64>(2)? as u32,
            ))
        })?;
        rows.collect::<rusqlite::Result<Vec<_>>>()
            .map_err(DbError::from)
    }
}
