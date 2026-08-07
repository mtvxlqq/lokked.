//! The `timer_presets` table: named timer configurations (countup, countdown
//! or pomodoro), either global (`subject_id = NULL`) or scoped to one subject.

use chrono::{DateTime, Utc};
use rusqlite::{params, OptionalExtension};
use uuid::Uuid;

use super::{Database, DbError};

/// A saved timer configuration, as stored in `timer_presets`.
#[derive(Debug, Clone, PartialEq)]
pub struct TimerPreset {
    pub id: String,
    /// `None` means a global preset, offered for every subject.
    pub subject_id: Option<String>,
    pub name: String,
    /// `'countup' | 'countdown' | 'pomodoro'` — validated by the caller
    /// (`core::timer::Mode` on the way in), stored as plain text.
    pub mode: String,
    pub work_seconds: i64,
    pub break_seconds: Option<i64>,
    pub long_break_seconds: Option<i64>,
    pub cycles_before_long: Option<i64>,
    pub auto_start_next: bool,
    pub is_default: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

fn row_to_preset(row: &rusqlite::Row<'_>) -> rusqlite::Result<TimerPreset> {
    Ok(TimerPreset {
        id: row.get("id")?,
        subject_id: row.get("subject_id")?,
        name: row.get("name")?,
        mode: row.get("mode")?,
        work_seconds: row.get("work_seconds")?,
        break_seconds: row.get("break_seconds")?,
        long_break_seconds: row.get("long_break_seconds")?,
        cycles_before_long: row.get("cycles_before_long")?,
        auto_start_next: row.get("auto_start_next")?,
        is_default: row.get("is_default")?,
        created_at: row.get("created_at")?,
        updated_at: row.get("updated_at")?,
        deleted_at: row.get("deleted_at")?,
    })
}

/// Fields needed to create a [`TimerPreset`]; grouped into one struct because
/// [`PresetRepo::create`] would otherwise take nine positional arguments.
pub struct NewPreset<'a> {
    pub subject_id: Option<&'a str>,
    pub name: &'a str,
    pub mode: &'a str,
    pub work_seconds: i64,
    pub break_seconds: Option<i64>,
    pub long_break_seconds: Option<i64>,
    pub cycles_before_long: Option<i64>,
    pub auto_start_next: bool,
    pub is_default: bool,
}

/// CRUD for `timer_presets`. Deletion is always soft, same as [`super::subjects::SubjectRepo`].
pub struct PresetRepo<'a> {
    db: &'a Database,
}

impl<'a> PresetRepo<'a> {
    pub fn new(db: &'a Database) -> Self {
        Self { db }
    }

    pub fn create(&self, new: NewPreset<'_>) -> Result<TimerPreset, DbError> {
        let id = Uuid::now_v7().to_string();
        let now = Utc::now();

        self.db.connection().execute(
            "INSERT INTO timer_presets
                (id, subject_id, name, mode, work_seconds, break_seconds, long_break_seconds,
                 cycles_before_long, auto_start_next, is_default, created_at, updated_at, deleted_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?11, NULL)",
            params![
                id,
                new.subject_id,
                new.name,
                new.mode,
                new.work_seconds,
                new.break_seconds,
                new.long_break_seconds,
                new.cycles_before_long,
                new.auto_start_next,
                new.is_default,
                now,
            ],
        )?;

        Ok(TimerPreset {
            id,
            subject_id: new.subject_id.map(str::to_string),
            name: new.name.to_string(),
            mode: new.mode.to_string(),
            work_seconds: new.work_seconds,
            break_seconds: new.break_seconds,
            long_break_seconds: new.long_break_seconds,
            cycles_before_long: new.cycles_before_long,
            auto_start_next: new.auto_start_next,
            is_default: new.is_default,
            created_at: now,
            updated_at: now,
            deleted_at: None,
        })
    }

    pub fn get(&self, id: &str) -> Result<Option<TimerPreset>, DbError> {
        self.db
            .connection()
            .query_row(
                "SELECT id, subject_id, name, mode, work_seconds, break_seconds,
                        long_break_seconds, cycles_before_long, auto_start_next, is_default,
                        created_at, updated_at, deleted_at
                 FROM timer_presets WHERE id = ?1",
                params![id],
                row_to_preset,
            )
            .optional()
            .map_err(DbError::from)
    }

    /// Presets not soft-deleted. Includes both global presets and ones
    /// scoped to a subject; callers filter by `subject_id` themselves.
    pub fn list(&self) -> Result<Vec<TimerPreset>, DbError> {
        let conn = self.db.connection();
        let mut stmt = conn.prepare(
            "SELECT id, subject_id, name, mode, work_seconds, break_seconds,
                    long_break_seconds, cycles_before_long, auto_start_next, is_default,
                    created_at, updated_at, deleted_at
             FROM timer_presets WHERE deleted_at IS NULL ORDER BY created_at",
        )?;
        let rows = stmt.query_map([], row_to_preset)?;
        rows.collect::<rusqlite::Result<Vec<_>>>()
            .map_err(DbError::from)
    }

    pub fn update(&self, id: &str, new: NewPreset<'_>) -> Result<(), DbError> {
        self.db.connection().execute(
            "UPDATE timer_presets
             SET subject_id = ?2, name = ?3, mode = ?4, work_seconds = ?5, break_seconds = ?6,
                 long_break_seconds = ?7, cycles_before_long = ?8, auto_start_next = ?9,
                 is_default = ?10, updated_at = ?11
             WHERE id = ?1",
            params![
                id,
                new.subject_id,
                new.name,
                new.mode,
                new.work_seconds,
                new.break_seconds,
                new.long_break_seconds,
                new.cycles_before_long,
                new.auto_start_next,
                new.is_default,
                Utc::now(),
            ],
        )?;
        Ok(())
    }

    pub fn soft_delete(&self, id: &str) -> Result<(), DbError> {
        let now = Utc::now();
        self.db.connection().execute(
            "UPDATE timer_presets SET deleted_at = ?2, updated_at = ?2 WHERE id = ?1",
            params![id, now],
        )?;
        Ok(())
    }
}
