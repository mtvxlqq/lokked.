//! Tests for bulk import: the plain-text format a student pastes in, and the
//! JSON export a set of lecture cards arrives in.
//!
//! Nothing here touches the database — parsing is a pure function of the text,
//! and the preview the import screen shows is exactly what these tests check.

use lokked_lib::core::import::{
    parse_lecture_json, parse_text, to_text, ImportOptions, ImportProblemKind,
};

fn default_options() -> ImportOptions {
    ImportOptions::default()
}

// --- текстовый формат ------------------------------------------------------

#[test]
fn two_cards_separated_by_the_card_separator() {
    let text = "\
Первообразная
---
Функция $F$, для которой $F'=f$
===
Интеграл
---
Множество всех первообразных";

    let preview = parse_text(text, &default_options());

    assert_eq!(preview.cards.len(), 2);
    assert_eq!(preview.cards[0].front, "Первообразная");
    assert_eq!(preview.cards[0].back, "Функция $F$, для которой $F'=f$");
    assert_eq!(preview.cards[1].front, "Интеграл");
    assert!(preview.problems.is_empty());
}

#[test]
fn a_third_section_becomes_the_hint() {
    let text = "Лицо\n---\nОборот\n---\nПодсказка";

    let preview = parse_text(text, &default_options());

    assert_eq!(preview.cards.len(), 1);
    assert_eq!(preview.cards[0].hint.as_deref(), Some("Подсказка"));
}

#[test]
fn the_text_of_a_card_keeps_its_own_line_breaks() {
    let text = "Теорема\n---\nПервая строка\n\nВторая строка\n$$F'(x)=f(x)$$";

    let preview = parse_text(text, &default_options());

    assert_eq!(
        preview.cards[0].back,
        "Первая строка\n\nВторая строка\n$$F'(x)=f(x)$$"
    );
}

#[test]
fn windows_line_endings_do_not_change_anything() {
    let text = "Лицо\r\n---\r\nОборот\r\n===\r\nВторое лицо\r\n---\r\nВторой оборот";

    let preview = parse_text(text, &default_options());

    assert_eq!(preview.cards.len(), 2);
    assert_eq!(preview.cards[0].back, "Оборот");
    assert_eq!(preview.cards[1].front, "Второе лицо");
}

#[test]
fn extra_separators_and_blank_blocks_are_skipped_quietly() {
    // Так выглядит текст, который редактировали руками: разделитель в начале,
    // два подряд в середине, один в конце.
    let text = "===\nЛицо\n---\nОборот\n===\n\n===\nВторое\n---\nОборот\n===\n";

    let preview = parse_text(text, &default_options());

    assert_eq!(preview.cards.len(), 2);
    assert!(preview.problems.is_empty());
}

#[test]
fn a_card_without_a_back_is_reported_rather_than_guessed_at() {
    let text = "Лицо\n---\nОборот\n===\nОдинокая карточка";

    let preview = parse_text(text, &default_options());

    assert_eq!(preview.cards.len(), 1);
    assert_eq!(preview.problems.len(), 1);
    assert_eq!(preview.problems[0].block, 2);
    assert_eq!(preview.problems[0].kind, ImportProblemKind::MissingBack);
}

#[test]
fn a_card_with_a_blank_side_is_reported() {
    let text = "Лицо\n---\n   \n===\n   \n---\nОборот";

    let preview = parse_text(text, &default_options());

    assert!(preview.cards.is_empty());
    assert_eq!(preview.problems.len(), 2);
    assert!(preview
        .problems
        .iter()
        .all(|problem| problem.kind == ImportProblemKind::BlankSide));
}

#[test]
fn too_many_sections_are_reported_with_how_many_there_were() {
    let text = "Лицо\n---\nОборот\n---\nПодсказка\n---\nЛишнее";

    let preview = parse_text(text, &default_options());

    assert!(preview.cards.is_empty());
    assert_eq!(
        preview.problems[0].kind,
        ImportProblemKind::TooManySides { found: 4 }
    );
}

#[test]
fn a_separator_inside_a_line_is_just_text() {
    // «5 === 5» на строке с другими символами — это содержимое карточки,
    // а не разделитель: иначе математику импортировать было бы нельзя.
    let text = "Тождество 5 === 5\n---\nВерно для любого --- дефиса";

    let preview = parse_text(text, &default_options());

    assert_eq!(preview.cards.len(), 1);
    assert_eq!(preview.cards[0].front, "Тождество 5 === 5");
    assert_eq!(preview.cards[0].back, "Верно для любого --- дефиса");
}

#[test]
fn separators_are_matched_after_trimming_the_line() {
    let text = "Лицо\n  ---  \nОборот";

    let preview = parse_text(text, &default_options());

    assert_eq!(preview.cards.len(), 1);
    assert_eq!(preview.cards[0].back, "Оборот");
}

#[test]
fn the_separators_can_be_something_else() {
    let options = ImportOptions::new("%%", "|").unwrap();
    let text = "Лицо\n|\nОборот\n%%\nВторое\n|\nОборот";

    let preview = parse_text(text, &options);

    assert_eq!(preview.cards.len(), 2);
}

#[test]
fn separators_have_to_be_usable() {
    assert!(ImportOptions::new("", "---").is_err());
    assert!(ImportOptions::new("===", "  ").is_err());
    // Один и тот же разделитель для карточек и для сторон разобрать нельзя.
    assert!(ImportOptions::new("---", "---").is_err());
}

#[test]
fn nothing_at_all_parses_into_nothing_at_all() {
    let preview = parse_text("   \n\n  ", &default_options());

    assert!(preview.cards.is_empty());
    assert!(preview.problems.is_empty());
}

// --- экспорт ---------------------------------------------------------------

#[test]
fn exported_text_parses_back_into_the_same_cards() {
    let text = "Лицо\n---\nОборот\n===\nВторое\n---\nОборот\n---\nПодсказка";
    let options = default_options();

    let parsed = parse_text(text, &options);
    let exported = to_text(&parsed.cards, &options);
    let reparsed = parse_text(&exported, &options);

    assert_eq!(reparsed.cards, parsed.cards);
}

// --- JSON с карточками лекций ----------------------------------------------

const LECTURE_JSON: &str = r#"{
  "meta": {
    "title": "Определения и теоремы, Терсенов",
    "source": "Терсенов А.С. Курс лекций по математическому анализу",
    "scope": "§ 25 — § 40"
  },
  "cards": [
    {
      "id": "25.01",
      "lecture": 25,
      "topic": "Неопределённый интеграл",
      "type": "определение",
      "title": "Первообразная функции",
      "statement": "Функция $F(x)$ называется **первообразной**, если $F'(x)=f(x)$.",
      "corrections": []
    },
    {
      "id": "25.02",
      "lecture": 25,
      "topic": "Неопределённый интеграл",
      "type": "теорема",
      "title": "Об общем виде первообразных",
      "statement": "Если $F$ — первообразная, то и $F+C$ тоже.",
      "corrections": ["исправлена опечатка"]
    }
  ]
}"#;

#[test]
fn the_lecture_json_becomes_cards() {
    let import = parse_lecture_json(LECTURE_JSON).unwrap();

    assert_eq!(import.preview.cards.len(), 2);
    assert_eq!(import.preview.cards[0].front, "Первообразная функции");
    assert!(import.preview.cards[0].back.contains("первообразной"));
    assert!(import.preview.problems.is_empty());
}

#[test]
fn the_type_topic_and_lecture_become_tags() {
    let import = parse_lecture_json(LECTURE_JSON).unwrap();

    assert_eq!(
        import.preview.cards[0].tags,
        vec!["определение", "Неопределённый интеграл", "лекция 25"]
    );
}

#[test]
fn the_deck_is_named_after_the_file_it_came_from() {
    let import = parse_lecture_json(LECTURE_JSON).unwrap();

    assert_eq!(
        import.preview.suggested_deck.as_deref(),
        Some("Определения и теоремы, Терсенов")
    );
    assert!(import
        .preview
        .suggested_description
        .as_deref()
        .unwrap()
        .contains("Терсенов А.С."));
}

#[test]
fn a_card_missing_its_statement_is_reported_and_the_rest_still_import() {
    let json = r#"{"cards":[
        {"title":"Есть","statement":"Оборот"},
        {"title":"Только лицо"},
        {"statement":"Только оборот"}
    ]}"#;

    let import = parse_lecture_json(json).unwrap();

    assert_eq!(import.preview.cards.len(), 1);
    assert_eq!(import.preview.problems.len(), 2);
    assert_eq!(import.preview.problems[0].block, 2);
    assert_eq!(
        import.preview.problems[0].kind,
        ImportProblemKind::MissingBack
    );
    assert_eq!(
        import.preview.problems[1].kind,
        ImportProblemKind::BlankSide
    );
}

#[test]
fn something_that_is_not_this_json_is_refused_with_a_reason() {
    assert!(parse_lecture_json("не json").is_err());
    assert!(parse_lecture_json(r#"{"meta":{}}"#).is_err());
    assert!(parse_lecture_json(r#"{"cards":"строка"}"#).is_err());
}
