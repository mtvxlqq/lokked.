//! Typed settings on top of the `settings` key-value table.
//!
//! The table stores strings; this module owns what those strings mean. It is
//! the only place that knows the keys and the accepted values, so both the
//! command layer and the tests speak the same vocabulary.
//!
//! Reading is deliberately forgiving and writing is not: a value the app
//! cannot read (an older row, a newer version's vocabulary, a hand-edited
//! database) falls back to the default rather than breaking the screen, while
//! a value coming from the settings screen is validated, because a bad one
//! there is a bug.

use std::fmt;

use chrono::TimeDelta;
use serde::{Deserialize, Serialize};

/// «Показывать только минуты» on the black screen. Stored as `1` / `0`.
pub const KEY_MINUTES_ONLY: &str = "zen.minutes_only";

/// Size of the digits on the black screen. Stored as a [`ZenFontSize`] slug.
pub const KEY_FONT_SIZE: &str = "zen.font_size";

/// «Гасить экран без движения» on the black screen. Stored as `1` / `0`.
pub const KEY_DIM_WHEN_IDLE: &str = "zen.dim_when_idle";

/// Where the study day starts, as a number of seconds after local midnight.
pub const KEY_DAY_START: &str = "day.start_offset_seconds";

/// How long one card lasts in a blitz, in seconds.
pub const KEY_BLITZ_SECONDS: &str = "blitz.seconds";

/// Why a setting from the UI was rejected.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SettingsError {
    UnknownFontSize(String),
    /// The day boundary was outside a day, or landed mid-minute.
    InvalidDayStart(i64),
    /// A blitz card was given an unusable amount of time.
    InvalidBlitzSeconds(i64),
}

impl fmt::Display for SettingsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownFontSize(value) => write!(f, "неизвестный размер шрифта: {value}"),
            Self::InvalidDayStart(seconds) => write!(
                f,
                "начало учебного дня должно быть временем суток с точностью до минуты, а не {seconds} с"
            ),
            Self::InvalidBlitzSeconds(seconds) => write!(
                f,
                "на карточку в блице нужно от {MIN_BLITZ_SECONDS} до {MAX_BLITZ_SECONDS} секунд, а не {seconds}"
            ),
        }
    }
}

impl std::error::Error for SettingsError {}

/// How large the digits on the black screen are drawn.
///
/// Named steps rather than a number of pixels: the black screen is one line
/// of digits that has to fit both a 380px phone and a monitor across the
/// room, so each step is a pair of sizes the frontend picks between at its
/// breakpoint. [`ZenFontSize::Normal`] is the size the design specifies.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ZenFontSize {
    Small,
    #[default]
    Normal,
    Large,
}

impl ZenFontSize {
    /// The slug stored in the table and sent to the frontend.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Small => "small",
            Self::Normal => "normal",
            Self::Large => "large",
        }
    }

    /// Parses a slug that came from the settings screen, rejecting anything
    /// else. Use [`ZenSettings::from_pairs`] for values read back out of the
    /// table, which must never fail.
    pub fn parse(raw: &str) -> Result<Self, SettingsError> {
        match raw {
            "small" => Ok(Self::Small),
            "normal" => Ok(Self::Normal),
            "large" => Ok(Self::Large),
            other => Err(SettingsError::UnknownFontSize(other.to_string())),
        }
    }
}

/// Everything the black screen reads out of the settings table.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ZenSettings {
    /// Show `1:07` instead of `1:07:24` — a ticking seconds digit is exactly
    /// the kind of movement the black screen exists to remove.
    pub minutes_only: bool,
    pub font_size: ZenFontSize,
    /// Fade the digits after a few seconds without a movement. On by
    /// default, because that is how the screen has always behaved — and off
    /// for the student who keeps the timer in the corner of their eye and
    /// wants to read it without touching anything.
    pub dim_when_idle: bool,
}

/// Not `#[derive(Default)]`: dimming defaults to on, and a derive would
/// quietly give it `false` — turning the black screen into a lamp for
/// everyone who never opened the settings screen.
impl Default for ZenSettings {
    fn default() -> Self {
        Self {
            minutes_only: false,
            font_size: ZenFontSize::default(),
            dim_when_idle: true,
        }
    }
}

impl ZenSettings {
    /// Reads the settings out of whatever the table holds. Unknown keys are
    /// ignored and unreadable values fall back to the default.
    pub fn from_pairs<'a>(pairs: impl IntoIterator<Item = (&'a str, &'a str)>) -> Self {
        let mut settings = Self::default();

        for (key, value) in pairs {
            match key {
                KEY_MINUTES_ONLY => settings.minutes_only = value == "1",
                KEY_FONT_SIZE => settings.font_size = ZenFontSize::parse(value).unwrap_or_default(),
                KEY_DIM_WHEN_IDLE => {
                    settings.dim_when_idle = match value {
                        "1" => true,
                        "0" => false,
                        _ => Self::default().dim_when_idle,
                    }
                }
                _ => {}
            }
        }

        settings
    }

    /// The rows to write for these settings.
    pub fn to_pairs(&self) -> [(&'static str, String); 3] {
        [
            (
                KEY_MINUTES_ONLY,
                if self.minutes_only { "1" } else { "0" }.to_string(),
            ),
            (KEY_FONT_SIZE, self.font_size.as_str().to_string()),
            (
                KEY_DIM_WHEN_IDLE,
                if self.dim_when_idle { "1" } else { "0" }.to_string(),
            ),
        ]
    }
}

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

/// Where a deck's blitz record is kept.
///
/// One key per deck rather than one row per record: a record is a single
/// number, and the settings table is exactly the place for single numbers
/// that have to survive a restart.
pub fn blitz_record_key(deck_id: &str) -> String {
    format!("blitz.best.{deck_id}")
}
