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

use serde::{Deserialize, Serialize};

/// «Показывать только минуты» on the black screen. Stored as `1` / `0`.
pub const KEY_MINUTES_ONLY: &str = "zen.minutes_only";

/// Size of the digits on the black screen. Stored as a [`ZenFontSize`] slug.
pub const KEY_FONT_SIZE: &str = "zen.font_size";

/// Why a setting from the UI was rejected.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SettingsError {
    UnknownFontSize(String),
}

impl fmt::Display for SettingsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownFontSize(value) => write!(f, "неизвестный размер шрифта: {value}"),
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct ZenSettings {
    /// Show `1:07` instead of `1:07:24` — a ticking seconds digit is exactly
    /// the kind of movement the black screen exists to remove.
    pub minutes_only: bool,
    pub font_size: ZenFontSize,
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
                _ => {}
            }
        }

        settings
    }

    /// The rows to write for these settings.
    pub fn to_pairs(&self) -> [(&'static str, String); 2] {
        [
            (
                KEY_MINUTES_ONLY,
                if self.minutes_only { "1" } else { "0" }.to_string(),
            ),
            (KEY_FONT_SIZE, self.font_size.as_str().to_string()),
        ]
    }
}
