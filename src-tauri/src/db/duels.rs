//! The `duels`, `duel_players` and `duel_answers` tables.
//!
//! A duel is stored apart from `sessions` and `reviews` on purpose: a guest's
//! answers belong to the evening, not to the owner's study record, and
//! keeping them here is what guarantees they never reach the statistics
//! screen or the card picker. The owner's own answers are written to
//! `reviews` too, by the command layer — they studied.
//!
//! `duel_answers` is append-only, like `reviews`: written once, never edited.
//! The scores on `duel_players` are the exception — they are recomputed from
//! the answers as a turn goes and rewritten at its end, so the summary needs
//! one query instead of a fold over every answer.

use chrono::{DateTime, Utc};
use rusqlite::{params, OptionalExtension};
use uuid::Uuid;

use super::{Database, DbError};

/// A duel as stored in `duels`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Duel {
    pub id: String,
    pub deck_id: String,
    pub day_key: String,
    pub started_at: DateTime<Utc>,
    /// `None` while the duel is still going.
    pub finished_at: Option<DateTime<Utc>>,
    pub cards: i64,
    pub seconds_per_card: i64,
}

/// A duel on its way into the table.
pub struct NewDuel<'a> {
    pub deck_id: &'a str,
    pub day_key: &'a str,
    pub started_at: DateTime<Utc>,
    pub cards: i64,
    pub seconds_per_card: i64,
}

/// One player of a stored duel, with what they scored.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoredPlayer {
    pub id: String,
    pub duel_id: String,
    pub name: String,
    pub position: i64,
    pub is_owner: bool,
    pub points: i64,
    pub correct: i64,
    pub best_streak: i64,
}

/// One answer of a duel.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DuelAnswer {
    pub id: String,
    pub duel_id: String,
    pub player_id: String,
    pub card_id: String,
    pub position: i64,
    pub result: String,
    pub correct: bool,
    pub total_ms: Option<i64>,
}

/// An answer on its way into the table.
pub struct NewDuelAnswer<'a> {
    pub duel_id: &'a str,
    pub player_id: &'a str,
    pub card_id: &'a str,
    pub position: i64,
    pub result: &'a str,
    pub correct: bool,
    pub total_ms: Option<i64>,
}

fn row_to_duel(row: &rusqlite::Row<'_>) -> rusqlite::Result<Duel> {
    Ok(Duel {
        id: row.get("id")?,
        deck_id: row.get("deck_id")?,
        day_key: row.get("day_key")?,
        started_at: row.get("started_at")?,
        finished_at: row.get("finished_at")?,
        cards: row.get("cards")?,
        seconds_per_card: row.get("seconds_per_card")?,
    })
}

fn row_to_player(row: &rusqlite::Row<'_>) -> rusqlite::Result<StoredPlayer> {
    Ok(StoredPlayer {
        id: row.get("id")?,
        duel_id: row.get("duel_id")?,
        name: row.get("name")?,
        position: row.get("position")?,
        is_owner: row.get::<_, i64>("is_owner")? != 0,
        points: row.get("points")?,
        correct: row.get("correct")?,
        best_streak: row.get("best_streak")?,
    })
}

fn row_to_answer(row: &rusqlite::Row<'_>) -> rusqlite::Result<DuelAnswer> {
    Ok(DuelAnswer {
        id: row.get("id")?,
        duel_id: row.get("duel_id")?,
        player_id: row.get("player_id")?,
        card_id: row.get("card_id")?,
        position: row.get("position")?,
        result: row.get("result")?,
        correct: row.get::<_, i64>("correct")? != 0,
        total_ms: row.get("total_ms")?,
    })
}

const DUEL_COLUMNS: &str = "id, deck_id, day_key, started_at, finished_at, cards, seconds_per_card";
const PLAYER_COLUMNS: &str = "id, duel_id, name, position, is_owner, points, correct, best_streak";
const ANSWER_COLUMNS: &str = "id, duel_id, player_id, card_id, position, result, correct, total_ms";

/// Reads and writes the three duel tables.
pub struct DuelRepo<'a> {
    db: &'a Database,
}

impl<'a> DuelRepo<'a> {
    pub fn new(db: &'a Database) -> Self {
        Self { db }
    }

    pub fn create(&self, new: NewDuel<'_>) -> Result<Duel, DbError> {
        let id = Uuid::now_v7().to_string();
        let now = Utc::now();

        self.db.connection().execute(
            "INSERT INTO duels (id, deck_id, day_key, started_at, finished_at, cards,
                                seconds_per_card, created_at, updated_at, deleted_at)
             VALUES (?1, ?2, ?3, ?4, NULL, ?5, ?6, ?7, ?7, NULL)",
            params![
                id,
                new.deck_id,
                new.day_key,
                new.started_at,
                new.cards,
                new.seconds_per_card,
                now,
            ],
        )?;

        Ok(Duel {
            id,
            deck_id: new.deck_id.to_string(),
            day_key: new.day_key.to_string(),
            started_at: new.started_at,
            finished_at: None,
            cards: new.cards,
            seconds_per_card: new.seconds_per_card,
        })
    }

    /// Adds one player and hands back the id their answers are filed under.
    pub fn add_player(
        &self,
        duel_id: &str,
        name: &str,
        position: i64,
        is_owner: bool,
    ) -> Result<String, DbError> {
        let id = Uuid::now_v7().to_string();
        let now = Utc::now();

        self.db.connection().execute(
            "INSERT INTO duel_players (id, duel_id, name, position, is_owner, points, correct,
                                       best_streak, created_at, updated_at, deleted_at)
             VALUES (?1, ?2, ?3, ?4, ?5, 0, 0, 0, ?6, ?6, NULL)",
            params![id, duel_id, name, position, is_owner as i64, now],
        )?;

        Ok(id)
    }

    pub fn record_answer(&self, new: NewDuelAnswer<'_>) -> Result<(), DbError> {
        self.db.connection().execute(
            "INSERT INTO duel_answers (id, duel_id, player_id, card_id, position, result,
                                       correct, total_ms, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                Uuid::now_v7().to_string(),
                new.duel_id,
                new.player_id,
                new.card_id,
                new.position,
                new.result,
                new.correct as i64,
                new.total_ms,
                Utc::now(),
            ],
        )?;

        Ok(())
    }

    /// Writes down what a player's turn came to.
    pub fn save_score(
        &self,
        player_id: &str,
        points: i64,
        correct: i64,
        best_streak: i64,
    ) -> Result<(), DbError> {
        self.db.connection().execute(
            "UPDATE duel_players SET points = ?2, correct = ?3, best_streak = ?4, updated_at = ?5
             WHERE id = ?1 AND deleted_at IS NULL",
            params![player_id, points, correct, best_streak, Utc::now()],
        )?;

        Ok(())
    }

    /// Closes the duel. A duel left halfway simply keeps `finished_at` empty.
    pub fn finish(&self, duel_id: &str, finished_at: DateTime<Utc>) -> Result<(), DbError> {
        self.db.connection().execute(
            "UPDATE duels SET finished_at = ?2, updated_at = ?3
             WHERE id = ?1 AND deleted_at IS NULL",
            params![duel_id, finished_at, Utc::now()],
        )?;

        Ok(())
    }

    pub fn get(&self, id: &str) -> Result<Option<Duel>, DbError> {
        self.db
            .connection()
            .query_row(
                &format!("SELECT {DUEL_COLUMNS} FROM duels WHERE id = ?1 AND deleted_at IS NULL"),
                params![id],
                row_to_duel,
            )
            .optional()
            .map_err(DbError::from)
    }

    /// The players of one duel, in turn order.
    pub fn players(&self, duel_id: &str) -> Result<Vec<StoredPlayer>, DbError> {
        let conn = self.db.connection();
        let mut stmt = conn.prepare(&format!(
            "SELECT {PLAYER_COLUMNS} FROM duel_players
             WHERE duel_id = ?1 AND deleted_at IS NULL
             ORDER BY position"
        ))?;
        let rows = stmt.query_map(params![duel_id], row_to_player)?;
        rows.collect::<rusqlite::Result<Vec<_>>>()
            .map_err(DbError::from)
    }

    /// Every answer of one duel, oldest first.
    pub fn answers(&self, duel_id: &str) -> Result<Vec<DuelAnswer>, DbError> {
        let conn = self.db.connection();
        let mut stmt = conn.prepare(&format!(
            "SELECT {ANSWER_COLUMNS} FROM duel_answers WHERE duel_id = ?1 ORDER BY created_at, id"
        ))?;
        let rows = stmt.query_map(params![duel_id], row_to_answer)?;
        rows.collect::<rusqlite::Result<Vec<_>>>()
            .map_err(DbError::from)
    }
}
