//! Tests for what a card and a deck are allowed to be.

use lokked_lib::core::card::{
    join_tags, normalize_hint, normalize_side, normalize_tags, split_tags, CardError, MAX_TAGS,
    MAX_TAG_LEN,
};
use lokked_lib::core::deck::{normalize_deck_name, normalize_description, DeckError, MAX_NAME_LEN};

// --- стороны карточки ------------------------------------------------------

#[test]
fn a_side_keeps_its_text_and_loses_the_whitespace_around_it() {
    assert_eq!(
        normalize_side("  Первообразная функции\n").unwrap(),
        "Первообразная функции"
    );
}

#[test]
fn a_side_keeps_the_line_breaks_inside_it() {
    // Формулировка теоремы — это абзацы и выключные формулы; схлопнуть их
    // значило бы испортить карточку.
    let statement = "Если $F$ — первообразная, то\n\n$$F'(x)=f(x).$$";

    assert_eq!(normalize_side(statement).unwrap(), statement);
}

#[test]
fn an_empty_side_is_refused() {
    assert_eq!(normalize_side(""), Err(CardError::EmptySide));
    assert_eq!(normalize_side("   \n\t "), Err(CardError::EmptySide));
}

#[test]
fn a_hint_is_optional_and_blank_counts_as_absent() {
    assert_eq!(
        normalize_hint(Some(" вспомни дифференциал ")).as_deref(),
        Some("вспомни дифференциал")
    );
    assert_eq!(normalize_hint(Some("   ")), None);
    assert_eq!(normalize_hint(None), None);
}

// --- теги ------------------------------------------------------------------

#[test]
fn tags_are_trimmed_and_blank_ones_dropped() {
    let tags = normalize_tags(&[
        " определение ".to_string(),
        "".to_string(),
        "лекция 25".to_string(),
    ])
    .unwrap();

    assert_eq!(tags, vec!["определение", "лекция 25"]);
}

#[test]
fn the_same_tag_twice_is_stored_once() {
    let tags = normalize_tags(&[
        "Определение".to_string(),
        "определение".to_string(),
        "ОПРЕДЕЛЕНИЕ".to_string(),
    ])
    .unwrap();

    // Первое написание и выигрывает: студент видит свой вариант, а не
    // приведённый к нижнему регистру.
    assert_eq!(tags, vec!["Определение"]);
}

#[test]
fn a_tag_with_a_comma_in_it_is_refused() {
    // Запятая — разделитель в хранимой строке, и тег с ней разъехался бы
    // на два при чтении.
    assert_eq!(
        normalize_tags(&["ряды, признаки".to_string()]),
        Err(CardError::TagWithComma("ряды, признаки".to_string()))
    );
}

#[test]
fn there_is_a_limit_on_tags_and_on_their_length() {
    let many: Vec<String> = (0..=MAX_TAGS).map(|n| format!("тег {n}")).collect();
    assert_eq!(
        normalize_tags(&many),
        Err(CardError::TooManyTags { max: MAX_TAGS })
    );

    let long = "т".repeat(MAX_TAG_LEN + 1);
    assert_eq!(
        normalize_tags(&[long]),
        Err(CardError::TagTooLong { max: MAX_TAG_LEN })
    );
}

#[test]
fn tags_survive_a_round_trip_through_the_stored_column() {
    let tags = vec!["определение".to_string(), "лекция 25".to_string()];

    let stored = join_tags(&tags);

    assert_eq!(stored.as_deref(), Some("определение,лекция 25"));
    assert_eq!(split_tags(stored.as_deref()), tags);
}

#[test]
fn a_card_without_tags_stores_nothing_rather_than_an_empty_string() {
    assert_eq!(join_tags(&[]), None);
    assert!(split_tags(None).is_empty());
    assert!(split_tags(Some("")).is_empty());
}

// --- колода ----------------------------------------------------------------

#[test]
fn a_deck_name_is_trimmed_and_cannot_be_empty() {
    assert_eq!(
        normalize_deck_name("  Математический анализ ").unwrap(),
        "Математический анализ"
    );
    assert_eq!(normalize_deck_name("  "), Err(DeckError::EmptyName));
}

#[test]
fn a_deck_name_has_a_length_limit() {
    let long = "к".repeat(MAX_NAME_LEN + 1);

    assert_eq!(
        normalize_deck_name(&long),
        Err(DeckError::NameTooLong { max: MAX_NAME_LEN })
    );
}

#[test]
fn a_deck_description_is_optional() {
    assert_eq!(
        normalize_description(Some(" § 25 — § 40 ")).as_deref(),
        Some("§ 25 — § 40")
    );
    assert_eq!(normalize_description(Some("  ")), None);
    assert_eq!(normalize_description(None), None);
}
