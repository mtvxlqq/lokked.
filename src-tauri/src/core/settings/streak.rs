//! What makes a day count towards the streak.

use serde::{Deserialize, Serialize};

use super::{SettingsError, KEY_STREAK_MINIMUM};

/// Shortest and longest daily minimum the streak allows, in minutes.
///
/// Below five minutes a day counts for opening the app; above four hours the
/// streak stops being something a bad day can survive.
pub const MIN_STREAK_MINUTES: i64 = 5;
pub const MAX_STREAK_MINUTES: i64 = 4 * 60;

/// How much study makes a day count towards the streak.
///
/// Stored in seconds, chosen in minutes: everything that consumes it works
/// in seconds, and the settings screen has no business in the difference.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct StreakSettings {
    pub min_seconds: i64,
}

impl Default for StreakSettings {
    fn default() -> Self {
        Self {
            min_seconds: crate::core::stats::streak::STREAK_MIN_SECONDS,
        }
    }
}

impl StreakSettings {
    /// Validates a value from the settings screen. Whole minutes only: the
    /// screen offers minutes, and a minimum of 612 seconds is a bug on the
    /// way in.
    pub fn new(min_seconds: i64) -> Result<Self, SettingsError> {
        let minutes = min_seconds / 60;
        if min_seconds % 60 != 0 || !(MIN_STREAK_MINUTES..=MAX_STREAK_MINUTES).contains(&minutes) {
            return Err(SettingsError::InvalidStreakMinimum(min_seconds));
        }

        Ok(Self { min_seconds })
    }

    /// Reads the value out of the table, falling back to the default.
    pub fn from_pairs<'a>(pairs: impl IntoIterator<Item = (&'a str, &'a str)>) -> Self {
        let mut settings = Self::default();

        for (key, value) in pairs {
            if key == KEY_STREAK_MINIMUM {
                settings = value
                    .parse::<i64>()
                    .ok()
                    .and_then(|seconds| Self::new(seconds).ok())
                    .unwrap_or_default();
            }
        }

        settings
    }

    pub fn to_pairs(&self) -> [(&'static str, String); 1] {
        [(KEY_STREAK_MINIMUM, self.min_seconds.to_string())]
    }

    /// The rules [`crate::core::stats::streak::streak_state`] counts by.
    pub fn rules(&self) -> crate::core::stats::streak::StreakRules {
        crate::core::stats::streak::StreakRules {
            min_seconds: self.min_seconds,
            ..Default::default()
        }
    }
}
