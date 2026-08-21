//! Tests for the settings vocabulary: how stored key-value pairs become the
//! typed settings the app works with, and what happens to values it does not
//! recognise.

use lokked_lib::core::settings::{
    SettingsError, ZenFontSize, ZenSettings, KEY_FONT_SIZE, KEY_MINUTES_ONLY,
};

#[test]
fn nothing_stored_yields_the_defaults() {
    let settings = ZenSettings::from_pairs([]);

    assert_eq!(settings, ZenSettings::default());
    assert!(!settings.minutes_only);
    assert_eq!(settings.font_size, ZenFontSize::Normal);
}

#[test]
fn minutes_only_is_stored_as_one_or_zero() {
    let on = ZenSettings::from_pairs([(KEY_MINUTES_ONLY, "1")]);
    let off = ZenSettings::from_pairs([(KEY_MINUTES_ONLY, "0")]);

    assert!(on.minutes_only);
    assert!(!off.minutes_only);
}

#[test]
fn a_value_that_is_neither_one_nor_zero_reads_as_off() {
    // The flag is off by default, and a row written by some future version
    // must not turn the black screen into something the student did not ask
    // for.
    for stored in ["", "нет", "yes", "2"] {
        assert!(!ZenSettings::from_pairs([(KEY_MINUTES_ONLY, stored)]).minutes_only);
    }
}

#[test]
fn each_font_size_reads_back_as_itself() {
    for size in [ZenFontSize::Small, ZenFontSize::Normal, ZenFontSize::Large] {
        let settings = ZenSettings::from_pairs([(KEY_FONT_SIZE, size.as_str())]);

        assert_eq!(settings.font_size, size);
    }
}

#[test]
fn an_unreadable_font_size_falls_back_to_the_default() {
    let settings = ZenSettings::from_pairs([(KEY_FONT_SIZE, "gigantic")]);

    assert_eq!(settings.font_size, ZenFontSize::Normal);
}

#[test]
fn keys_from_elsewhere_in_the_table_are_ignored() {
    let settings = ZenSettings::from_pairs([
        ("day.start_offset_seconds", "14400"),
        (KEY_MINUTES_ONLY, "1"),
    ]);

    assert!(settings.minutes_only);
    assert_eq!(settings.font_size, ZenFontSize::Normal);
}

#[test]
fn settings_survive_a_round_trip_through_the_stored_pairs() {
    let settings = ZenSettings {
        minutes_only: true,
        font_size: ZenFontSize::Large,
    };

    let stored = settings.to_pairs();
    let read_back = ZenSettings::from_pairs(
        stored
            .iter()
            .map(|(key, value)| (*key, value.as_str()))
            .collect::<Vec<_>>(),
    );

    assert_eq!(read_back, settings);
}

#[test]
fn parsing_a_font_size_from_the_ui_rejects_what_it_does_not_know() {
    // Reading the table is forgiving; taking a value from the settings screen
    // is not — a typo there is a bug worth surfacing, not a silent default.
    assert_eq!(ZenFontSize::parse("large"), Ok(ZenFontSize::Large));
    assert_eq!(
        ZenFontSize::parse("gigantic"),
        Err(SettingsError::UnknownFontSize("gigantic".to_string()))
    );
}
