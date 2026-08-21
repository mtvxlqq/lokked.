//! Naming and rotating the database backups.
//!
//! The copy itself is I/O and lives in [`crate::db::backup`]; what belongs
//! here is the part with rules — what a copy is called, and which of them
//! are old enough to delete. Both are pure functions of a timestamp and a
//! list of names.

use chrono::{DateTime, Utc};

/// How many copies are kept. A week of daily launches: far enough back to
/// notice something went wrong, small enough that the folder stays a folder
/// and not an archive.
pub const BACKUPS_KEPT: usize = 7;

/// The prefix and suffix a copy of ours is recognised by.
const PREFIX: &str = "lokked-";
const SUFFIX: &str = ".sqlite3";

/// What the copy taken at `moment` is called.
///
/// The timestamp is written so that sorting the names alphabetically sorts
/// them chronologically — that is what makes the rotation below a `sort` and
/// nothing more.
pub fn backup_name(moment: DateTime<Utc>) -> String {
    format!("{PREFIX}{}{SUFFIX}", moment.format("%Y%m%d-%H%M%S"))
}

/// Which of `names` should be deleted so that only the newest `keep` remain.
///
/// Only files this module could have created are considered: anything else
/// in the folder belongs to the student and is never touched.
pub fn stale(names: &[String], keep: usize) -> Vec<String> {
    let mut ours: Vec<&String> = names
        .iter()
        .filter(|name| is_backup(name))
        .collect::<Vec<_>>();

    ours.sort();

    let extra = ours.len().saturating_sub(keep);

    ours.into_iter().take(extra).cloned().collect()
}

/// Whether this file name is one of ours.
fn is_backup(name: &str) -> bool {
    name.starts_with(PREFIX) && name.ends_with(SUFFIX) && name.len() > PREFIX.len() + SUFFIX.len()
}
