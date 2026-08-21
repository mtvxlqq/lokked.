//! The `subjects` table: what a student studies, with a colour and an icon.

use chrono::{DateTime, Utc};
use rusqlite::{params, OptionalExtension};
use uuid::Uuid;

use super::{Database, DbError};

/// A subject a student studies, as stored in `subjects`.
#[derive(Debug, Clone, PartialEq)]
pub struct Subject {
    pub id: String,
    pub name: String,
    pub color: Option<String>,
    pub icon: Option<String>,
    pub position: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

fn row_to_subject(row: &rusqlite::Row<'_>) -> rusqlite::Result<Subject> {
    Ok(Subject {
        id: row.get("id")?,
        name: row.get("name")?,
        color: row.get("color")?,
        icon: row.get("icon")?,
        position: row.get("position")?,
        created_at: row.get("created_at")?,
        updated_at: row.get("updated_at")?,
        deleted_at: row.get("deleted_at")?,
    })
}

/// CRUD for `subjects`. Deletion is always soft: [`SubjectRepo::soft_delete`]
/// sets `deleted_at` rather than removing the row, so [`SubjectRepo::list`]
/// filters it out while [`SubjectRepo::get`] can still find it.
pub struct SubjectRepo<'a> {
    db: &'a Database,
}

impl<'a> SubjectRepo<'a> {
    pub fn new(db: &'a Database) -> Self {
        Self { db }
    }

    pub fn create(
        &self,
        name: &str,
        color: Option<&str>,
        icon: Option<&str>,
        position: i64,
    ) -> Result<Subject, DbError> {
        let id = Uuid::now_v7().to_string();
        let now = Utc::now();

        self.db.connection().execute(
            "INSERT INTO subjects (id, name, color, icon, position, created_at, updated_at, deleted_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?6, NULL)",
            params![id, name, color, icon, position, now],
        )?;

        Ok(Subject {
            id,
            name: name.to_string(),
            color: color.map(str::to_string),
            icon: icon.map(str::to_string),
            position,
            created_at: now,
            updated_at: now,
            deleted_at: None,
        })
    }

    pub fn get(&self, id: &str) -> Result<Option<Subject>, DbError> {
        self.db
            .connection()
            .query_row(
                "SELECT id, name, color, icon, position, created_at, updated_at, deleted_at
                 FROM subjects WHERE id = ?1",
                params![id],
                row_to_subject,
            )
            .optional()
            .map_err(DbError::from)
    }

    /// Subjects not soft-deleted, ordered for display.
    pub fn list(&self) -> Result<Vec<Subject>, DbError> {
        let conn = self.db.connection();
        let mut stmt = conn.prepare(
            "SELECT id, name, color, icon, position, created_at, updated_at, deleted_at
             FROM subjects WHERE deleted_at IS NULL ORDER BY position",
        )?;
        let rows = stmt.query_map([], row_to_subject)?;
        rows.collect::<rusqlite::Result<Vec<_>>>()
            .map_err(DbError::from)
    }

    /// One past the largest position ever used, counting soft-deleted rows.
    ///
    /// Deliberately not «number of live subjects»: reusing the position of a
    /// deleted subject would make two rows sort identically the moment
    /// deletion ever becomes undoable, and the number is only ever used to
    /// append.
    pub fn next_position(&self) -> Result<i64, DbError> {
        self.db
            .connection()
            .query_row(
                "SELECT COALESCE(MAX(position), -1) + 1 FROM subjects",
                [],
                |row| row.get(0),
            )
            .map_err(DbError::from)
    }

    pub fn update(
        &self,
        id: &str,
        name: &str,
        color: Option<&str>,
        icon: Option<&str>,
        position: i64,
    ) -> Result<(), DbError> {
        self.db.connection().execute(
            "UPDATE subjects SET name = ?2, color = ?3, icon = ?4, position = ?5, updated_at = ?6
             WHERE id = ?1",
            params![id, name, color, icon, position, Utc::now()],
        )?;
        Ok(())
    }

    pub fn soft_delete(&self, id: &str) -> Result<(), DbError> {
        let now = Utc::now();
        self.db.connection().execute(
            "UPDATE subjects SET deleted_at = ?2, updated_at = ?2 WHERE id = ?1",
            params![id, now],
        )?;
        Ok(())
    }
}
