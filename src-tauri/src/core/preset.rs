//! Validation rules for a timer preset, and the bridge from a stored preset to
//! a runnable [`Mode`].
//!
//! A preset row is stringly typed by necessity — SQLite has no sum types — so
//! every path out of the database goes through [`validate`], which turns the
//! loose row into a [`ValidPreset`] whose fields are guaranteed consistent
//! with its kind. [`ValidPreset::to_mode`] is then total: it cannot fail.

use std::fmt;

use chrono::TimeDelta;

use crate::core::timer::Mode;

/// Longest accepted preset name, in characters.
pub const MAX_NAME_LEN: usize = 40;

/// No single phase may be longer than a day. This is a sanity bound, not a
/// product decision: it keeps a stray «25000 минут» from producing a timer
/// whose arithmetic overflows anything downstream.
pub const MAX_PHASE_SECONDS: i64 = 24 * 60 * 60;

/// Most work phases before a long break. Beyond this the "long break" stops
/// meaning anything.
pub const MAX_CYCLES: i64 = 12;

/// What a preset times. The stored `mode` column holds [`PresetKind::as_str`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PresetKind {
    CountUp,
    CountDown,
    Pomodoro,
}

impl PresetKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::CountUp => "countup",
            Self::CountDown => "countdown",
            Self::Pomodoro => "pomodoro",
        }
    }

    pub fn parse(raw: &str) -> Result<Self, PresetError> {
        match raw {
            "countup" => Ok(Self::CountUp),
            "countdown" => Ok(Self::CountDown),
            "pomodoro" => Ok(Self::Pomodoro),
            other => Err(PresetError::UnknownMode(other.to_string())),
        }
    }
}

/// Why a preset was rejected.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PresetError {
    EmptyName,
    NameTooLong {
        max: usize,
    },
    UnknownMode(String),
    /// A duration this kind needs was absent.
    Missing {
        field: &'static str,
    },
    /// A duration was zero or negative.
    NotPositive {
        field: &'static str,
    },
    /// A duration exceeded [`MAX_PHASE_SECONDS`], or the cycle count exceeded
    /// [`MAX_CYCLES`].
    OutOfRange {
        field: &'static str,
        max: i64,
    },
}

impl fmt::Display for PresetError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyName => write!(f, "название пресета не может быть пустым"),
            Self::NameTooLong { max } => write!(f, "название пресета длиннее {max} символов"),
            Self::UnknownMode(mode) => write!(f, "неизвестный режим таймера: {mode}"),
            Self::Missing { field } => write!(f, "не задано поле {field}"),
            Self::NotPositive { field } => write!(f, "поле {field} должно быть больше нуля"),
            Self::OutOfRange { field, max } => write!(f, "поле {field} больше максимума {max}"),
        }
    }
}

impl std::error::Error for PresetError {}

/// A preset as it arrives from the UI or out of a database row: fields that
/// may or may not belong to the chosen kind, in whatever state the caller had
/// them.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PresetDraft<'a> {
    pub name: &'a str,
    pub mode: &'a str,
    pub work_seconds: i64,
    pub break_seconds: Option<i64>,
    pub long_break_seconds: Option<i64>,
    pub cycles_before_long: Option<i64>,
    pub auto_start_next: bool,
}

/// A preset whose fields are known to match its kind.
///
/// Fields irrelevant to the kind are cleared to `None` rather than carried
/// along: a countdown preset that remembers a stale `break_seconds` from when
/// it was a Pomodoro is a bug waiting to surface after the next edit.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidPreset {
    pub name: String,
    pub kind: PresetKind,
    /// Length of a work phase. Always `0` for [`PresetKind::CountUp`], which
    /// counts without a target.
    pub work_seconds: i64,
    pub break_seconds: Option<i64>,
    pub long_break_seconds: Option<i64>,
    pub cycles_before_long: Option<i64>,
    /// Only meaningful for Pomodoro; forced to `false` otherwise, since the
    /// other kinds have no next phase to start.
    pub auto_start_next: bool,
}

fn positive(value: i64, field: &'static str) -> Result<i64, PresetError> {
    if value <= 0 {
        return Err(PresetError::NotPositive { field });
    }
    if value > MAX_PHASE_SECONDS {
        return Err(PresetError::OutOfRange {
            field,
            max: MAX_PHASE_SECONDS,
        });
    }
    Ok(value)
}

fn required(value: Option<i64>, field: &'static str) -> Result<i64, PresetError> {
    positive(value.ok_or(PresetError::Missing { field })?, field)
}

/// Checks a draft and normalises it for its kind.
pub fn validate(draft: PresetDraft<'_>) -> Result<ValidPreset, PresetError> {
    let name = draft.name.trim();
    if name.is_empty() {
        return Err(PresetError::EmptyName);
    }
    if name.chars().count() > MAX_NAME_LEN {
        return Err(PresetError::NameTooLong { max: MAX_NAME_LEN });
    }

    let kind = PresetKind::parse(draft.mode)?;

    let valid = match kind {
        PresetKind::CountUp => ValidPreset {
            name: name.to_string(),
            kind,
            work_seconds: 0,
            break_seconds: None,
            long_break_seconds: None,
            cycles_before_long: None,
            auto_start_next: false,
        },
        PresetKind::CountDown => ValidPreset {
            name: name.to_string(),
            kind,
            work_seconds: positive(draft.work_seconds, "work_seconds")?,
            break_seconds: None,
            long_break_seconds: None,
            cycles_before_long: None,
            auto_start_next: false,
        },
        PresetKind::Pomodoro => {
            let cycles = draft
                .cycles_before_long
                .ok_or(PresetError::Missing {
                    field: "cycles_before_long",
                })
                .and_then(|n| {
                    if n <= 0 {
                        Err(PresetError::NotPositive {
                            field: "cycles_before_long",
                        })
                    } else if n > MAX_CYCLES {
                        Err(PresetError::OutOfRange {
                            field: "cycles_before_long",
                            max: MAX_CYCLES,
                        })
                    } else {
                        Ok(n)
                    }
                })?;

            ValidPreset {
                name: name.to_string(),
                kind,
                work_seconds: positive(draft.work_seconds, "work_seconds")?,
                break_seconds: Some(required(draft.break_seconds, "break_seconds")?),
                long_break_seconds: Some(required(draft.long_break_seconds, "long_break_seconds")?),
                cycles_before_long: Some(cycles),
                auto_start_next: draft.auto_start_next,
            }
        }
    };

    Ok(valid)
}

impl ValidPreset {
    /// The runnable [`Mode`] this preset describes.
    ///
    /// Total by construction: [`validate`] has already guaranteed every field
    /// the chosen kind needs is present and positive.
    pub fn to_mode(&self) -> Mode {
        match self.kind {
            PresetKind::CountUp => Mode::CountUp,
            PresetKind::CountDown => Mode::CountDown {
                target: TimeDelta::seconds(self.work_seconds),
            },
            PresetKind::Pomodoro => Mode::Pomodoro {
                work: TimeDelta::seconds(self.work_seconds),
                short_break: TimeDelta::seconds(self.break_seconds.unwrap_or_default()),
                long_break: TimeDelta::seconds(self.long_break_seconds.unwrap_or_default()),
                cycles_before_long_break: self.cycles_before_long.unwrap_or(1) as u32,
                auto_start_next: self.auto_start_next,
            },
        }
    }
}

/// A stored preset, reduced to what decides whether it is the one to run.
///
/// Deliberately not the whole preset: choosing does not need the durations,
/// and a borrowed view keeps this function usable straight from a list of
/// database rows without copying them.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PresetChoice<'a> {
    pub id: &'a str,
    /// `None` for a global preset.
    pub subject_id: Option<&'a str>,
    pub is_default: bool,
}

/// Which preset the «Старт» button should run for `subject_id`.
///
/// In order: the subject's own default, any preset of that subject, the
/// global default, any global preset. A preset attached to a subject beats a
/// global default even when it is not marked as default — attaching it to the
/// subject was already a statement about that subject.
///
/// `None` means no preset applies; the caller decides what to do about it
/// (Lokked starts a plain stopwatch).
pub fn select_preset<'a>(presets: &'a [PresetChoice<'a>], subject_id: &str) -> Option<&'a str> {
    let of_subject = |preset: &&PresetChoice<'a>| preset.subject_id == Some(subject_id);
    let global = |preset: &&PresetChoice<'a>| preset.subject_id.is_none();

    let pick = |scope: &dyn Fn(&&PresetChoice<'a>) -> bool| {
        presets
            .iter()
            .find(|preset| scope(preset) && preset.is_default)
            .or_else(|| presets.iter().find(|preset| scope(preset)))
    };

    pick(&of_subject).or_else(|| pick(&global)).map(|p| p.id)
}
