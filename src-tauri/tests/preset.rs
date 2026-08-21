//! Tests for `core::preset`: which stored preset rows are consistent with
//! their kind, and what runnable `Mode` each one becomes.

use chrono::TimeDelta;
use lokked_lib::core::preset::{
    validate, PresetDraft, PresetError, PresetKind, MAX_CYCLES, MAX_NAME_LEN, MAX_PHASE_SECONDS,
};
use lokked_lib::core::timer::Mode;

/// A Pomodoro draft with every field filled in — tests below knock out the
/// one field they are about.
fn pomodoro() -> PresetDraft<'static> {
    PresetDraft {
        name: "Классический",
        mode: "pomodoro",
        work_seconds: 25 * 60,
        break_seconds: Some(5 * 60),
        long_break_seconds: Some(15 * 60),
        cycles_before_long: Some(4),
        auto_start_next: true,
    }
}

#[test]
fn parses_and_prints_every_kind() {
    for kind in [
        PresetKind::CountUp,
        PresetKind::CountDown,
        PresetKind::Pomodoro,
    ] {
        assert_eq!(PresetKind::parse(kind.as_str()).unwrap(), kind);
    }
}

#[test]
fn rejects_an_unknown_mode() {
    let draft = PresetDraft {
        mode: "stopwatch",
        ..pomodoro()
    };
    assert_eq!(
        validate(draft).unwrap_err(),
        PresetError::UnknownMode("stopwatch".to_string())
    );
}

#[test]
fn trims_the_name_and_checks_its_length_in_characters() {
    let draft = PresetDraft {
        name: "  Классический  ",
        ..pomodoro()
    };
    assert_eq!(validate(draft).unwrap().name, "Классический");

    let at_limit = "я".repeat(MAX_NAME_LEN);
    assert!(validate(PresetDraft {
        name: &at_limit,
        ..pomodoro()
    })
    .is_ok());

    let over_limit = "я".repeat(MAX_NAME_LEN + 1);
    assert_eq!(
        validate(PresetDraft {
            name: &over_limit,
            ..pomodoro()
        })
        .unwrap_err(),
        PresetError::NameTooLong { max: MAX_NAME_LEN }
    );
}

#[test]
fn rejects_a_blank_name() {
    assert_eq!(
        validate(PresetDraft {
            name: "   ",
            ..pomodoro()
        })
        .unwrap_err(),
        PresetError::EmptyName
    );
}

#[test]
fn countup_ignores_every_duration_it_does_not_use() {
    let draft = PresetDraft {
        name: "Свободно",
        mode: "countup",
        // Left over from when this preset was a Pomodoro.
        ..pomodoro()
    };

    let valid = validate(draft).unwrap();
    assert_eq!(valid.kind, PresetKind::CountUp);
    assert_eq!(valid.work_seconds, 0);
    assert_eq!(valid.break_seconds, None);
    assert_eq!(valid.long_break_seconds, None);
    assert_eq!(valid.cycles_before_long, None);
    assert!(!valid.auto_start_next);
    assert_eq!(valid.to_mode(), Mode::CountUp);
}

#[test]
fn countdown_keeps_only_its_work_phase() {
    let draft = PresetDraft {
        name: "45 минут",
        mode: "countdown",
        work_seconds: 45 * 60,
        ..pomodoro()
    };

    let valid = validate(draft).unwrap();
    assert_eq!(valid.work_seconds, 45 * 60);
    assert_eq!(valid.break_seconds, None);
    assert_eq!(valid.long_break_seconds, None);
    assert_eq!(valid.cycles_before_long, None);
    assert!(!valid.auto_start_next);
    assert_eq!(
        valid.to_mode(),
        Mode::CountDown {
            target: TimeDelta::seconds(45 * 60)
        }
    );
}

#[test]
fn countdown_needs_a_positive_work_phase() {
    for seconds in [0, -60] {
        assert_eq!(
            validate(PresetDraft {
                mode: "countdown",
                work_seconds: seconds,
                ..pomodoro()
            })
            .unwrap_err(),
            PresetError::NotPositive {
                field: "work_seconds"
            }
        );
    }
}

#[test]
fn pomodoro_keeps_every_field_and_becomes_a_pomodoro_mode() {
    let valid = validate(pomodoro()).unwrap();

    assert_eq!(valid.kind, PresetKind::Pomodoro);
    assert!(valid.auto_start_next);
    assert_eq!(
        valid.to_mode(),
        Mode::Pomodoro {
            work: TimeDelta::seconds(25 * 60),
            short_break: TimeDelta::seconds(5 * 60),
            long_break: TimeDelta::seconds(15 * 60),
            cycles_before_long_break: 4,
            auto_start_next: true,
        }
    );
}

#[test]
fn pomodoro_needs_every_duration() {
    assert_eq!(
        validate(PresetDraft {
            break_seconds: None,
            ..pomodoro()
        })
        .unwrap_err(),
        PresetError::Missing {
            field: "break_seconds"
        }
    );
    assert_eq!(
        validate(PresetDraft {
            long_break_seconds: None,
            ..pomodoro()
        })
        .unwrap_err(),
        PresetError::Missing {
            field: "long_break_seconds"
        }
    );
    assert_eq!(
        validate(PresetDraft {
            cycles_before_long: None,
            ..pomodoro()
        })
        .unwrap_err(),
        PresetError::Missing {
            field: "cycles_before_long"
        }
    );
}

#[test]
fn pomodoro_durations_must_be_positive() {
    assert_eq!(
        validate(PresetDraft {
            break_seconds: Some(0),
            ..pomodoro()
        })
        .unwrap_err(),
        PresetError::NotPositive {
            field: "break_seconds"
        }
    );
    assert_eq!(
        validate(PresetDraft {
            long_break_seconds: Some(-1),
            ..pomodoro()
        })
        .unwrap_err(),
        PresetError::NotPositive {
            field: "long_break_seconds"
        }
    );
    assert_eq!(
        validate(PresetDraft {
            cycles_before_long: Some(0),
            ..pomodoro()
        })
        .unwrap_err(),
        PresetError::NotPositive {
            field: "cycles_before_long"
        }
    );
}

#[test]
fn a_phase_may_be_a_whole_day_but_no_longer() {
    assert!(validate(PresetDraft {
        mode: "countdown",
        work_seconds: MAX_PHASE_SECONDS,
        ..pomodoro()
    })
    .is_ok());

    assert_eq!(
        validate(PresetDraft {
            mode: "countdown",
            work_seconds: MAX_PHASE_SECONDS + 1,
            ..pomodoro()
        })
        .unwrap_err(),
        PresetError::OutOfRange {
            field: "work_seconds",
            max: MAX_PHASE_SECONDS
        }
    );
}

#[test]
fn the_cycle_count_is_capped() {
    assert!(validate(PresetDraft {
        cycles_before_long: Some(MAX_CYCLES),
        ..pomodoro()
    })
    .is_ok());

    assert_eq!(
        validate(PresetDraft {
            cycles_before_long: Some(MAX_CYCLES + 1),
            ..pomodoro()
        })
        .unwrap_err(),
        PresetError::OutOfRange {
            field: "cycles_before_long",
            max: MAX_CYCLES
        }
    );
}

#[test]
fn validating_is_idempotent() {
    let once = validate(pomodoro()).unwrap();
    let twice = validate(PresetDraft {
        name: &once.name,
        mode: once.kind.as_str(),
        work_seconds: once.work_seconds,
        break_seconds: once.break_seconds,
        long_break_seconds: once.long_break_seconds,
        cycles_before_long: once.cycles_before_long,
        auto_start_next: once.auto_start_next,
    })
    .unwrap();

    assert_eq!(once, twice);
}

// --- выбор пресета для запуска ------------------------------------------

use lokked_lib::core::preset::{select_preset, PresetChoice};

fn choice<'a>(id: &'a str, subject_id: Option<&'a str>, is_default: bool) -> PresetChoice<'a> {
    PresetChoice {
        id,
        subject_id,
        is_default,
    }
}

#[test]
fn without_any_preset_there_is_nothing_to_pick() {
    assert_eq!(select_preset(&[], "algebra"), None);
}

#[test]
fn the_subjects_own_default_wins() {
    let presets = [
        choice("global-default", None, true),
        choice("algebra-plain", Some("algebra"), false),
        choice("algebra-default", Some("algebra"), true),
    ];

    assert_eq!(select_preset(&presets, "algebra"), Some("algebra-default"));
}

#[test]
fn any_preset_of_the_subject_beats_a_global_one() {
    // Attaching a preset to a subject is already a statement about that
    // subject; a global default should not override it.
    let presets = [
        choice("global-default", None, true),
        choice("algebra-plain", Some("algebra"), false),
    ];

    assert_eq!(select_preset(&presets, "algebra"), Some("algebra-plain"));
}

#[test]
fn a_subject_with_nothing_of_its_own_falls_back_to_the_global_default() {
    let presets = [
        choice("global-plain", None, false),
        choice("global-default", None, true),
        choice("physics-default", Some("physics"), true),
    ];

    assert_eq!(select_preset(&presets, "algebra"), Some("global-default"));
}

#[test]
fn with_no_default_anywhere_the_first_global_preset_is_used() {
    let presets = [
        choice("global-first", None, false),
        choice("global-second", None, false),
    ];

    assert_eq!(select_preset(&presets, "algebra"), Some("global-first"));
}

#[test]
fn another_subjects_presets_are_never_picked() {
    let presets = [choice("physics-default", Some("physics"), true)];

    assert_eq!(select_preset(&presets, "algebra"), None);
}
