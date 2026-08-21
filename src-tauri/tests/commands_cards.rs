//! Tests for the deck, card and import commands: what the editor may store,
//! and what a bulk import does to a deck.

use lokked_lib::commands::cards::{self, CardInput};
use lokked_lib::commands::decks::{self, DeckInput};
use lokked_lib::commands::import::{self, ImportFormat};
use lokked_lib::commands::subjects::{self, SubjectInput};
use lokked_lib::commands::ErrorKind;
use lokked_lib::core::import::{ImportOptions, ParsedCard};
use lokked_lib::db::Database;

fn new_db() -> Database {
    Database::open_in_memory().expect("in-memory database should open")
}

fn deck(db: &Database, name: &str) -> String {
    decks::create(
        db,
        DeckInput {
            subject_id: None,
            name: name.to_string(),
            description: None,
        },
    )
    .unwrap()
    .id
}

fn card(front: &str, back: &str) -> CardInput {
    CardInput {
        front: front.to_string(),
        back: back.to_string(),
        hint: None,
        tags: Vec::new(),
    }
}

// --- колоды ----------------------------------------------------------------

#[test]
fn a_deck_can_be_filed_under_a_subject() {
    let db = new_db();
    let subject = subjects::create(
        &db,
        SubjectInput {
            name: "Математический анализ".to_string(),
            color: None,
            icon: None,
        },
    )
    .unwrap();

    let deck = decks::create(
        &db,
        DeckInput {
            subject_id: Some(subject.id.clone()),
            name: "Терсенов, § 25 — § 40".to_string(),
            description: Some("  лекции  ".to_string()),
        },
    )
    .unwrap();

    assert_eq!(deck.subject_id.as_deref(), Some(subject.id.as_str()));
    assert_eq!(deck.description.as_deref(), Some("лекции"));
    assert_eq!(deck.card_count, 0);
}

#[test]
fn a_deck_under_a_subject_that_is_gone_is_refused() {
    let db = new_db();

    let error = decks::create(
        &db,
        DeckInput {
            subject_id: Some("нет такого предмета".to_string()),
            name: "Колода".to_string(),
            description: None,
        },
    )
    .unwrap_err();

    assert_eq!(error.kind, ErrorKind::NotFound);
}

#[test]
fn a_deck_without_a_name_is_refused() {
    let db = new_db();

    let error = decks::create(
        &db,
        DeckInput {
            subject_id: None,
            name: "   ".to_string(),
            description: None,
        },
    )
    .unwrap_err();

    assert_eq!(error.kind, ErrorKind::Validation);
}

#[test]
fn the_deck_list_carries_how_many_cards_each_one_holds() {
    let db = new_db();
    let full = deck(&db, "С карточками");
    deck(&db, "Пустая");
    cards::create(&db, &full, card("Лицо", "Оборот")).unwrap();
    cards::create(&db, &full, card("Второе", "Оборот")).unwrap();

    let listed = decks::list(&db).unwrap();

    assert_eq!(listed.len(), 2);
    assert_eq!(listed[0].card_count, 2);
    assert_eq!(listed[1].card_count, 0);
}

#[test]
fn a_deleted_deck_disappears_from_the_list_and_cannot_be_deleted_twice() {
    let db = new_db();
    let id = deck(&db, "Колода");

    decks::delete(&db, &id).unwrap();

    assert!(decks::list(&db).unwrap().is_empty());
    assert_eq!(
        decks::delete(&db, &id).unwrap_err().kind,
        ErrorKind::NotFound
    );
}

// --- карточки --------------------------------------------------------------

#[test]
fn a_card_keeps_its_formula_exactly_as_written() {
    let db = new_db();
    let id = deck(&db, "Колода");
    let statement = "Функция $F(x)$ называется **первообразной**, если\n$$F'(x)=f(x).$$";

    let created = cards::create(&db, &id, card("Первообразная", statement)).unwrap();

    assert_eq!(created.back, statement);
}

#[test]
fn a_card_with_an_empty_side_is_refused() {
    let db = new_db();
    let id = deck(&db, "Колода");

    let error = cards::create(&db, &id, card("Лицо", "   ")).unwrap_err();

    assert_eq!(error.kind, ErrorKind::Validation);
    assert!(cards::list(&db, &id).unwrap().is_empty());
}

#[test]
fn tags_come_back_as_a_list_not_as_a_string() {
    let db = new_db();
    let id = deck(&db, "Колода");

    let created = cards::create(
        &db,
        &id,
        CardInput {
            tags: vec![
                " определение ".to_string(),
                "лекция 25".to_string(),
                "определение".to_string(),
            ],
            ..card("Лицо", "Оборот")
        },
    )
    .unwrap();

    assert_eq!(created.tags, vec!["определение", "лекция 25"]);
    assert_eq!(cards::list(&db, &id).unwrap()[0].tags, created.tags);
}

#[test]
fn editing_a_card_replaces_what_it_holds() {
    let db = new_db();
    let id = deck(&db, "Колода");
    let created = cards::create(&db, &id, card("Было", "старое")).unwrap();

    let updated = cards::update(
        &db,
        &created.id,
        CardInput {
            hint: Some(" вспомни дифференциал ".to_string()),
            ..card("Стало", "новое")
        },
    )
    .unwrap();

    assert_eq!(updated.front, "Стало");
    assert_eq!(updated.hint.as_deref(), Some("вспомни дифференциал"));
}

#[test]
fn a_card_moves_between_decks_without_losing_anything() {
    let db = new_db();
    let from = deck(&db, "Откуда");
    let to = deck(&db, "Куда");
    let created = cards::create(&db, &from, card("Лицо", "Оборот")).unwrap();

    let moved = cards::move_to_deck(&db, &created.id, &to).unwrap();

    assert_eq!(moved.deck_id, to);
    assert_eq!(moved.front, "Лицо");
    assert!(cards::list(&db, &from).unwrap().is_empty());
}

#[test]
fn a_deleted_card_is_gone_from_its_deck() {
    let db = new_db();
    let id = deck(&db, "Колода");
    let created = cards::create(&db, &id, card("Лицо", "Оборот")).unwrap();

    cards::delete(&db, &created.id).unwrap();

    assert!(cards::list(&db, &id).unwrap().is_empty());
    assert_eq!(
        cards::delete(&db, &created.id).unwrap_err().kind,
        ErrorKind::NotFound
    );
}

#[test]
fn cards_of_a_deck_that_is_gone_cannot_be_listed_or_added_to() {
    let db = new_db();

    assert_eq!(
        cards::list(&db, "нет такой колоды").unwrap_err().kind,
        ErrorKind::NotFound
    );
    assert_eq!(
        cards::create(&db, "нет такой колоды", card("Лицо", "Оборот"))
            .unwrap_err()
            .kind,
        ErrorKind::NotFound
    );
}

// --- импорт и экспорт ------------------------------------------------------

#[test]
fn the_format_is_recognised_from_the_text_itself() {
    let options = ImportOptions::default();

    let text = import::preview("Лицо\n---\nОборот", &options).unwrap();
    let json = import::preview(
        r#"{"cards":[{"title":"Лицо","statement":"Оборот"}]}"#,
        &options,
    )
    .unwrap();

    assert_eq!(text.format, ImportFormat::Text);
    assert_eq!(json.format, ImportFormat::LectureJson);
    assert_eq!(text.preview.cards.len(), 1);
    assert_eq!(json.preview.cards.len(), 1);
}

#[test]
fn a_preview_writes_nothing() {
    let db = new_db();
    let id = deck(&db, "Колода");

    import::preview("Лицо\n---\nОборот", &ImportOptions::default()).unwrap();

    assert!(cards::list(&db, &id).unwrap().is_empty());
}

#[test]
fn importing_writes_the_previewed_cards_into_the_deck() {
    let db = new_db();
    let id = deck(&db, "Колода");
    let report = import::preview(
        "Первообразная\n---\n$F'=f$\n===\nИнтеграл\n---\nмножество первообразных",
        &ImportOptions::default(),
    )
    .unwrap();

    let written = import::commit(&db, &id, &report.preview.cards).unwrap();

    let stored = cards::list(&db, &id).unwrap();
    assert_eq!(written, 2);
    assert_eq!(stored.len(), 2);
    assert_eq!(stored[0].front, "Первообразная");
    assert_eq!(stored[1].back, "множество первообразных");
}

#[test]
fn an_imported_card_is_validated_like_one_typed_by_hand() {
    let db = new_db();
    let id = deck(&db, "Колода");
    let cards_with_a_bad_tag = vec![ParsedCard {
        front: "Лицо".to_string(),
        back: "Оборот".to_string(),
        hint: None,
        tags: vec!["ряды, признаки".to_string()],
    }];

    let error = import::commit(&db, &id, &cards_with_a_bad_tag).unwrap_err();

    assert_eq!(error.kind, ErrorKind::Validation);
    assert!(cards::list(&db, &id).unwrap().is_empty());
}

#[test]
fn one_bad_card_stops_the_whole_import() {
    // Иначе колода окажется наполовину заполненной, и разобраться, какая
    // половина доехала, будет уже нельзя.
    let db = new_db();
    let id = deck(&db, "Колода");
    let mixed = vec![
        ParsedCard {
            front: "Хорошая".to_string(),
            back: "карточка".to_string(),
            hint: None,
            tags: Vec::new(),
        },
        ParsedCard {
            front: "   ".to_string(),
            back: "карточка".to_string(),
            hint: None,
            tags: Vec::new(),
        },
    ];

    assert!(import::commit(&db, &id, &mixed).is_err());
    assert!(cards::list(&db, &id).unwrap().is_empty());
}

#[test]
fn the_lecture_json_becomes_a_deck_with_tagged_cards() {
    let db = new_db();
    let id = deck(&db, "Матанализ");
    let json = r#"{
        "meta": {"title": "Терсенов", "source": "лекции", "scope": "§ 25 — § 40"},
        "cards": [{
            "lecture": 25,
            "topic": "Неопределённый интеграл",
            "type": "определение",
            "title": "Первообразная функции",
            "statement": "Функция $F(x)$, для которой $F'(x)=f(x)$."
        }]
    }"#;

    let report = import::preview(json, &ImportOptions::default()).unwrap();
    import::commit(&db, &id, &report.preview.cards).unwrap();

    let stored = cards::list(&db, &id).unwrap();
    assert_eq!(stored.len(), 1);
    assert_eq!(stored[0].front, "Первообразная функции");
    assert_eq!(
        stored[0].tags,
        vec!["определение", "Неопределённый интеграл", "лекция 25"]
    );
    assert_eq!(report.preview.suggested_deck.as_deref(), Some("Терсенов"));
}

#[test]
fn a_deck_exports_into_text_that_imports_back_unchanged() {
    let db = new_db();
    let source = deck(&db, "Откуда");
    let target = deck(&db, "Куда");
    cards::create(
        &db,
        &source,
        CardInput {
            hint: Some("подсказка".to_string()),
            ..card("Первообразная", "$F'=f$")
        },
    )
    .unwrap();
    cards::create(&db, &source, card("Интеграл", "множество первообразных")).unwrap();

    let exported = import::export(&db, &source, &ImportOptions::default()).unwrap();
    let report = import::preview(&exported, &ImportOptions::default()).unwrap();
    import::commit(&db, &target, &report.preview.cards).unwrap();

    let there = cards::list(&db, &source).unwrap();
    let back = cards::list(&db, &target).unwrap();
    assert_eq!(back.len(), there.len());
    assert_eq!(back[0].front, there[0].front);
    assert_eq!(back[0].hint, there[0].hint);
    assert_eq!(back[1].back, there[1].back);
}

#[test]
fn importing_into_a_deck_that_is_gone_is_refused() {
    let db = new_db();
    let cards = vec![ParsedCard {
        front: "Лицо".to_string(),
        back: "Оборот".to_string(),
        hint: None,
        tags: Vec::new(),
    }];

    assert_eq!(
        import::commit(&db, "нет такой колоды", &cards)
            .unwrap_err()
            .kind,
        ErrorKind::NotFound
    );
}
