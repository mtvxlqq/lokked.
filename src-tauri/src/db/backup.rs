//! Keeping a few days of the database around.
//!
//! One copy per launch, the newest [`BACKUPS_KEPT`] kept. The rules — what a
//! copy is called and which of them are stale — live in
//! [`crate::core::backup`]; this module is the part that touches the disk.
//!
//! Nothing here is allowed to stop the app: a failed backup is reported to
//! the caller, which logs it and carries on. Not being able to write a copy
//! is a worse Tuesday, not a reason to refuse to open the timer.

use std::fs;
use std::path::{Path, PathBuf};

use chrono::{DateTime, Utc};

use crate::core::backup::{backup_name, stale, BACKUPS_KEPT};

use super::{Database, DbError};

/// Where the copies live, relative to the app's data directory.
pub const DIRECTORY: &str = "backups";

/// Takes one copy and deletes whatever that pushes out of the window.
///
/// Returns the copy that was written.
pub fn rotate(db: &Database, dir: &Path, now: DateTime<Utc>) -> Result<PathBuf, DbError> {
    fs::create_dir_all(dir).map_err(|err| DbError::Backup(err.to_string()))?;

    let path = dir.join(backup_name(now));
    // Копия за ту же секунду уже есть — значит приложение запустили дважды
    // подряд; `VACUUM INTO` на существующий файл откажет, и это верно:
    // перезаписывать чужую копию нам незачем.
    if !path.exists() {
        db.backup_to(&path)?;
    }

    prune(dir, BACKUPS_KEPT)?;

    Ok(path)
}

/// Deletes copies beyond the newest `keep`.
///
/// Returns what was deleted. Files that are not ours stay where they are.
pub fn prune(dir: &Path, keep: usize) -> Result<Vec<PathBuf>, DbError> {
    let names: Vec<String> = fs::read_dir(dir)
        .map_err(|err| DbError::Backup(err.to_string()))?
        .filter_map(|entry| entry.ok())
        .filter_map(|entry| entry.file_name().into_string().ok())
        .collect();

    let mut removed = Vec::new();
    for name in stale(&names, keep) {
        let path = dir.join(name);
        fs::remove_file(&path).map_err(|err| DbError::Backup(err.to_string()))?;
        removed.push(path);
    }

    Ok(removed)
}
