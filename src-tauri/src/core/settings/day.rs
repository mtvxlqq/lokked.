//! Where the student's study day begins.

use chrono::TimeDelta;
use serde::{Deserialize, Serialize};

use super::{SettingsError, KEY_DAY_START};

/// Seconds in a day, the exclusive upper bound for a day boundary.
const DAY_SECONDS: i64 = 24 * 60 * 60;

/// Where the student's study day begins.
///
/// Stored as an offset from local midnight, not as `"04:00"`: everything that
/// consumes it — [`crate::core::dayline`] — works in [`TimeDelta`], and a
/// number needs no parsing rules of its own.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct DaySettings {
    pub start_offset_seconds: i64,
}

impl DaySettings {
    /// Validates a boundary coming from the settings screen. Whole minutes
    /// only, and inside a day: a study day beginning at 25:00 or at 04:00:37
    /// is a bug on the way in, not something to store and puzzle over later.
    pub fn new(start_offset_seconds: i64) -> Result<Self, SettingsError> {
        if !(0..DAY_SECONDS).contains(&start_offset_seconds) || start_offset_seconds % 60 != 0 {
            return Err(SettingsError::InvalidDayStart(start_offset_seconds));
        }

        Ok(Self {
            start_offset_seconds,
        })
    }

    /// Reads the boundary out of the table, falling back to midnight for
    /// anything unreadable.
    pub fn from_pairs<'a>(pairs: impl IntoIterator<Item = (&'a str, &'a str)>) -> Self {
        let mut settings = Self::default();

        for (key, value) in pairs {
            if key == KEY_DAY_START {
                settings = value
                    .parse::<i64>()
                    .ok()
                    .and_then(|seconds| Self::new(seconds).ok())
                    .unwrap_or_default();
            }
        }

        settings
    }

    /// The row to write for this boundary.
    pub fn to_pairs(&self) -> [(&'static str, String); 1] {
        [(KEY_DAY_START, self.start_offset_seconds.to_string())]
    }

    /// The offset in the form [`crate::core::dayline`] takes.
    pub fn start_offset(&self) -> TimeDelta {
        TimeDelta::seconds(self.start_offset_seconds)
    }
}
