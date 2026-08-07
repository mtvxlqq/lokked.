//! The `sessions` table: completed (or interrupted) study sessions.
//!
//! Append-only, per CLAUDE.md rule 3: rows are written once and never
//! updated or deleted, so [`SessionRepo`] deliberately offers no `update` or
//! `soft_delete` — unlike [`super::subjects::SubjectRepo`] and
//! [`super::presets::PresetRepo`], which do. `sessions` also has no
//! `deleted_at` column, matching the schema in
//! `src-tauri/migrations/0001_init.sql`.

use chrono::{DateTime, Utc};
use rusqlite::params;
use uuid::Uuid;

use super::{Database, DbError};

/// A recorded study session, as stored in `sessions`.
#[derive(Debug, Clone, PartialEq)]
pub struct Session {
    pub id: String,
    pub subject_id: String,
    pub preset_id: Option<String>,
    pub mode: String,
    /// `'work' | 'break' | 'long_break'`.
    pub phase: String,
    pub started_at: DateTime<Utc>,
    pub ended_at: DateTime<Utc>,
    /// 'YYYY-MM-DD', from `core::dayline::split_by_day` — a session
    /// spanning a day boundary is written as one row per
    /// [`core::dayline::Segment`], not split here.
    pub day_key: String,
    pub active_seconds: i64,
    pub paused_seconds: i64,
    pub planned_seconds: Option<i64>,
    pub completed: bool,
    pub interruptions: i64,
    pub device_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Fields needed to record a [`Session`].
pub struct NewSession<'a> {
    pub subject_id: &'a str,
    pub preset_id: Option<&'a str>,
    pub mode: &'a str,
    pub phase: &'a str,
    pub started_at: DateTime<Utc>,
    pub ended_at: DateTime<Utc>,
    pub day_key: &'a str,
    pub active_seconds: i64,
    pub paused_seconds: i64,
    pub planned_seconds: Option<i64>,
    pub completed: bool,
    pub interruptions: i64,
    pub device_id: Option<&'a str>,
}

fn row_to_session(row: &rusqlite::Row<'_>) -> rusqlite::Result<Session> {
    Ok(Session {
        id: row.get("id")?,
        subject_id: row.get("subject_id")?,
        preset_id: row.get("preset_id")?,
        mode: row.get("mode")?,
        phase: row.get("phase")?,
        started_at: row.get("started_at")?,
        ended_at: row.get("ended_at")?,
        day_key: row.get("day_key")?,
        active_seconds: row.get("active_seconds")?,
        paused_seconds: row.get("paused_seconds")?,
        planned_seconds: row.get("planned_seconds")?,
        completed: row.get("completed")?,
        interruptions: row.get("interruptions")?,
        device_id: row.get("device_id")?,
        created_at: row.get("created_at")?,
        updated_at: row.get("updated_at")?,
    })
}

/// Append-only access to `sessions`: write once with [`SessionRepo::create`],
/// read back by day or by subject. No update, no delete.
pub struct SessionRepo<'a> {
    db: &'a Database,
}

impl<'a> SessionRepo<'a> {
    pub fn new(db: &'a Database) -> Self {
        Self { db }
    }

    pub fn create(&self, new: NewSession<'_>) -> Result<Session, DbError> {
        let id = Uuid::now_v7().to_string();
        let now = Utc::now();

        self.db.connection().execute(
            "INSERT INTO sessions
                (id, subject_id, preset_id, mode, phase, started_at, ended_at, day_key,
                 active_seconds, paused_seconds, planned_seconds, completed, interruptions,
                 device_id, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?15)",
            params![
                id,
                new.subject_id,
                new.preset_id,
                new.mode,
                new.phase,
                new.started_at,
                new.ended_at,
                new.day_key,
                new.active_seconds,
                new.paused_seconds,
                new.planned_seconds,
                new.completed,
                new.interruptions,
                new.device_id,
                now,
            ],
        )?;

        Ok(Session {
            id,
            subject_id: new.subject_id.to_string(),
            preset_id: new.preset_id.map(str::to_string),
            mode: new.mode.to_string(),
            phase: new.phase.to_string(),
            started_at: new.started_at,
            ended_at: new.ended_at,
            day_key: new.day_key.to_string(),
            active_seconds: new.active_seconds,
            paused_seconds: new.paused_seconds,
            planned_seconds: new.planned_seconds,
            completed: new.completed,
            interruptions: new.interruptions,
            device_id: new.device_id.map(str::to_string),
            created_at: now,
            updated_at: now,
        })
    }

    /// All sessions recorded for one study day, oldest first.
    pub fn list_for_day(&self, day_key: &str) -> Result<Vec<Session>, DbError> {
        let conn = self.db.connection();
        let mut stmt = conn.prepare(
            "SELECT id, subject_id, preset_id, mode, phase, started_at, ended_at, day_key,
                    active_seconds, paused_seconds, planned_seconds, completed, interruptions,
                    device_id, created_at, updated_at
             FROM sessions WHERE day_key = ?1 ORDER BY started_at",
        )?;
        let rows = stmt.query_map(params![day_key], row_to_session)?;
        rows.collect::<rusqlite::Result<Vec<_>>>()
            .map_err(DbError::from)
    }

    /// All sessions recorded for one subject, oldest first.
    pub fn list_for_subject(&self, subject_id: &str) -> Result<Vec<Session>, DbError> {
        let conn = self.db.connection();
        let mut stmt = conn.prepare(
            "SELECT id, subject_id, preset_id, mode, phase, started_at, ended_at, day_key,
                    active_seconds, paused_seconds, planned_seconds, completed, interruptions,
                    device_id, created_at, updated_at
             FROM sessions WHERE subject_id = ?1 ORDER BY started_at",
        )?;
        let rows = stmt.query_map(params![subject_id], row_to_session)?;
        rows.collect::<rusqlite::Result<Vec<_>>>()
            .map_err(DbError::from)
    }
}
