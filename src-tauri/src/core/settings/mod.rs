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
//!
//! Split by what is being set: [`zen`] for the black screen, [`day`] for the
//! study-day boundary, [`cards`] for the two knobs a run of cards has, and
//! [`streak`] for what makes a day count. The keys and the rejection reasons
//! stay here, because they are the vocabulary all four share.

use std::fmt;

pub mod cards;
pub mod day;
pub mod streak;
pub mod zen;

pub use cards::{
    blitz_record_key, AdaptiveSettings, BlitzSettings, DEFAULT_AGGRESSIVENESS,
    DEFAULT_BLITZ_SECONDS, MAX_AGGRESSIVENESS_EXPONENT, MAX_BLITZ_SECONDS, MIN_BLITZ_SECONDS,
};
pub use day::DaySettings;
pub use streak::{StreakSettings, MAX_STREAK_MINUTES, MIN_STREAK_MINUTES};
pub use zen::{ZenFontSize, ZenSettings};

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

/// How strongly the card picker leans towards the weak cards, in percent.
pub const KEY_AGGRESSIVENESS: &str = "cards.aggressiveness";

/// How much study makes a day count towards the streak, in seconds.
pub const KEY_STREAK_MINIMUM: &str = "streak.min_seconds";

/// Why a setting from the UI was rejected.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SettingsError {
    UnknownFontSize(String),
    /// The day boundary was outside a day, or landed mid-minute.
    InvalidDayStart(i64),
    /// A blitz card was given an unusable amount of time.
    InvalidBlitzSeconds(i64),
    /// The picker's aggressiveness was outside the slider.
    InvalidAggressiveness(i64),
    /// The streak's daily minimum was unusable.
    InvalidStreakMinimum(i64),
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
            Self::InvalidAggressiveness(percent) => write!(
                f,
                "перекос в сторону слабых задаётся числом от 0 до 100, а не {percent}"
            ),
            Self::InvalidStreakMinimum(seconds) => write!(
                f,
                "дневной минимум серии — от {MIN_STREAK_MINUTES} до {MAX_STREAK_MINUTES} минут, а не {seconds} с"
            ),
        }
    }
}

impl std::error::Error for SettingsError {}
