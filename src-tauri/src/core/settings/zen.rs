//! The black screen's look: how large the digits are and whether they fade.

use serde::{Deserialize, Serialize};

use super::{SettingsError, KEY_DIM_WHEN_IDLE, KEY_FONT_SIZE, KEY_MINUTES_ONLY};

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
