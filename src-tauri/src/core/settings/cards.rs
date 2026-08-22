//! The two knobs a run of cards has: how long a blitz card lasts, and how
//! hard the picker leans towards the cards going badly.

use serde::{Deserialize, Serialize};

use super::{SettingsError, KEY_AGGRESSIVENESS, KEY_BLITZ_SECONDS};

/// Shortest and longest a blitz card may last.
///
/// Below five seconds there is no time to read the card, above two minutes
/// it is not a blitz any more.
pub const MIN_BLITZ_SECONDS: i64 = 5;
pub const MAX_BLITZ_SECONDS: i64 = 120;

/// The default: long enough to recall a formulation, short enough to hurry.
pub const DEFAULT_BLITZ_SECONDS: i64 = 20;

/// How long one card lasts in a blitz.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct BlitzSettings {
    pub seconds: i64,
}

impl Default for BlitzSettings {
    fn default() -> Self {
        Self {
            seconds: DEFAULT_BLITZ_SECONDS,
        }
    }
}

impl BlitzSettings {
    /// Validates a value from the settings screen.
    pub fn new(seconds: i64) -> Result<Self, SettingsError> {
        if !(MIN_BLITZ_SECONDS..=MAX_BLITZ_SECONDS).contains(&seconds) {
            return Err(SettingsError::InvalidBlitzSeconds(seconds));
        }

        Ok(Self { seconds })
    }

    /// Reads the value out of the table, falling back to the default.
    pub fn from_pairs<'a>(pairs: impl IntoIterator<Item = (&'a str, &'a str)>) -> Self {
        let mut settings = Self::default();

        for (key, value) in pairs {
            if key == KEY_BLITZ_SECONDS {
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
        [(KEY_BLITZ_SECONDS, self.seconds.to_string())]
    }
}

/// The strongest lean towards the weak cards the slider allows.
///
/// The value is an exponent the weights are raised to: `0` flattens them all
/// to one — a plain shuffle — `1` uses them as computed, and `2` squares the
/// gaps, so a card going badly comes up many times more often than one that
/// is not. Above two a run turns into the same three cards over and over.
pub const MAX_AGGRESSIVENESS_EXPONENT: f64 = 2.0;

/// Where the slider sits by default: the weights as they are computed.
pub const DEFAULT_AGGRESSIVENESS: i64 = 50;

/// How strongly the card picker leans towards the cards going badly.
///
/// Stored as a percentage of [`MAX_AGGRESSIVENESS_EXPONENT`] rather than as
/// the exponent itself: it is a slider on the settings screen, and a whole
/// number survives being written to the table and read back without any
/// question of formatting.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct AdaptiveSettings {
    pub aggressiveness: i64,
}

impl Default for AdaptiveSettings {
    fn default() -> Self {
        Self {
            aggressiveness: DEFAULT_AGGRESSIVENESS,
        }
    }
}

impl AdaptiveSettings {
    /// Validates a value from the settings screen.
    pub fn new(aggressiveness: i64) -> Result<Self, SettingsError> {
        if !(0..=100).contains(&aggressiveness) {
            return Err(SettingsError::InvalidAggressiveness(aggressiveness));
        }

        Ok(Self { aggressiveness })
    }

    /// Reads the value out of the table, falling back to the default.
    pub fn from_pairs<'a>(pairs: impl IntoIterator<Item = (&'a str, &'a str)>) -> Self {
        let mut settings = Self::default();

        for (key, value) in pairs {
            if key == KEY_AGGRESSIVENESS {
                settings = value
                    .parse::<i64>()
                    .ok()
                    .and_then(|percent| Self::new(percent).ok())
                    .unwrap_or_default();
            }
        }

        settings
    }

    pub fn to_pairs(&self) -> [(&'static str, String); 1] {
        [(KEY_AGGRESSIVENESS, self.aggressiveness.to_string())]
    }

    /// The exponent [`crate::core::scheduler::weights::weight`] takes.
    pub fn exponent(&self) -> f64 {
        MAX_AGGRESSIVENESS_EXPONENT * self.aggressiveness as f64 / 100.0
    }
}

/// Where a deck's blitz record is kept.
///
/// One key per deck rather than one row per record: a record is a single
/// number, and the settings table is exactly the place for single numbers
/// that have to survive a restart.
pub fn blitz_record_key(deck_id: &str) -> String {
    format!("blitz.best.{deck_id}")
}
