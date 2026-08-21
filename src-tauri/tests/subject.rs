//! Tests for `core::subject`: what counts as a storable subject name and
//! colour, and how a new subject picks its colour from the palette.

use lokked_lib::core::subject::{
    normalize_color, normalize_name, palette_slug, SubjectError, MAX_NAME_LEN, PALETTE_SIZE,
};

#[test]
fn trims_surrounding_whitespace() {
    assert_eq!(normalize_name("  Алгебра \n").unwrap(), "Алгебра");
}

#[test]
fn keeps_interior_whitespace_verbatim() {
    let name = "Теория вероятностей и статистика";
    assert_eq!(normalize_name(name).unwrap(), name);
}

#[test]
fn rejects_a_blank_name() {
    assert_eq!(normalize_name("").unwrap_err(), SubjectError::EmptyName);
    assert_eq!(
        normalize_name("   \t\n ").unwrap_err(),
        SubjectError::EmptyName
    );
}

#[test]
fn measures_length_in_characters_not_bytes() {
    // 40 Cyrillic characters are 80 bytes, and must still be accepted.
    let cyrillic = "я".repeat(40);
    assert_eq!(cyrillic.len(), 80);
    assert_eq!(normalize_name(&cyrillic).unwrap(), cyrillic);

    let at_limit = "я".repeat(MAX_NAME_LEN);
    assert!(normalize_name(&at_limit).is_ok());

    let over_limit = "я".repeat(MAX_NAME_LEN + 1);
    assert_eq!(
        normalize_name(&over_limit).unwrap_err(),
        SubjectError::NameTooLong { max: MAX_NAME_LEN }
    );
}

#[test]
fn length_is_checked_after_trimming() {
    let padded = format!("  {}  ", "я".repeat(MAX_NAME_LEN));
    assert!(normalize_name(&padded).is_ok());
}

#[test]
fn accepts_every_palette_slug() {
    for n in 1..=PALETTE_SIZE {
        let slug = format!("subject-{n}");
        assert_eq!(normalize_color(Some(&slug)).unwrap(), Some(slug));
    }
}

#[test]
fn treats_a_missing_or_blank_colour_as_none() {
    assert_eq!(normalize_color(None).unwrap(), None);
    assert_eq!(normalize_color(Some("")).unwrap(), None);
    assert_eq!(normalize_color(Some("   ")).unwrap(), None);
}

#[test]
fn rejects_colours_outside_the_palette() {
    for bad in [
        "subject-0",
        "subject-9",
        "subject-",
        "subject-1x",
        "subject--1",
        "#7e9cc4",
        "red",
    ] {
        assert_eq!(
            normalize_color(Some(bad)).unwrap_err(),
            SubjectError::UnknownColor(bad.to_string()),
            "expected {bad} to be rejected"
        );
    }
}

#[test]
fn palette_slugs_cycle_through_the_palette() {
    assert_eq!(palette_slug(0), "subject-1");
    assert_eq!(
        palette_slug(PALETTE_SIZE - 1),
        format!("subject-{PALETTE_SIZE}")
    );
    // The ninth subject starts the palette over rather than running off it.
    assert_eq!(palette_slug(PALETTE_SIZE), "subject-1");
    assert_eq!(palette_slug(PALETTE_SIZE * 3 + 2), "subject-3");
}

#[test]
fn every_generated_slug_is_accepted_back() {
    for index in 0..PALETTE_SIZE * 2 {
        let slug = palette_slug(index);
        assert_eq!(normalize_color(Some(&slug)).unwrap(), Some(slug));
    }
}
