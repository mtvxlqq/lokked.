//! Dev tool: load a card file straight into a database, without the UI.
//!
//! The app can import the same file from the cards screen; this is for
//! seeding a database from the command line — a fresh install, a test
//! fixture, or a file too big to want to click through.
//!
//! ```sh
//! cargo run --example import_cards -- \
//!     --db ~/.local/share/com.lokked.app/lokked.sqlite3 \
//!     --file cards.json \
//!     --subject "Математический анализ" \
//!     [--deck "Название колоды"] [--dry-run]
//! ```
//!
//! Both formats the app understands are accepted, and which one a file is in
//! is decided by its content. A deck of the same name is reused rather than
//! duplicated; `--dry-run` parses and reports without writing anything.

use std::collections::HashMap;
use std::process::ExitCode;

use lokked_lib::commands::decks::{self, DeckInput};
use lokked_lib::commands::import::{self};
use lokked_lib::core::import::ImportOptions;
use lokked_lib::db::subjects::SubjectRepo;
use lokked_lib::db::Database;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut options: HashMap<String, String> = HashMap::new();
    let mut dry_run = false;

    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--dry-run" => {
                dry_run = true;
                index += 1;
            }
            flag if flag.starts_with("--") && index + 1 < args.len() => {
                options.insert(
                    flag.trim_start_matches("--").to_string(),
                    args[index + 1].clone(),
                );
                index += 2;
            }
            other => {
                eprintln!("не понимаю аргумент: {other}");
                return ExitCode::FAILURE;
            }
        }
    }

    let (Some(db_path), Some(file)) = (options.get("db"), options.get("file")) else {
        eprintln!("нужны --db <путь к базе> и --file <файл с карточками>");
        return ExitCode::FAILURE;
    };

    let raw = match std::fs::read_to_string(file) {
        Ok(raw) => raw,
        Err(err) => {
            eprintln!("не читается {file}: {err}");
            return ExitCode::FAILURE;
        }
    };

    let db = match Database::open_at(db_path) {
        Ok(db) => db,
        Err(err) => {
            eprintln!("не открывается база {db_path}: {err}");
            return ExitCode::FAILURE;
        }
    };

    let report = match import::preview(&raw, &ImportOptions::default()) {
        Ok(report) => report,
        Err(err) => {
            eprintln!("файл не разобрался: {}", err.message);
            return ExitCode::FAILURE;
        }
    };

    println!("формат: {:?}", report.format);
    println!("распознано карточек: {}", report.preview.cards.len());
    for problem in &report.preview.problems {
        println!("  блок {}: {}", problem.block, problem.kind);
    }

    if dry_run {
        println!("--dry-run: ничего не записано");
        return ExitCode::SUCCESS;
    }

    let deck_name = options
        .get("deck")
        .cloned()
        .or_else(|| report.preview.suggested_deck.clone())
        .unwrap_or_else(|| "Импорт".to_string());

    let subject_id = match options.get("subject") {
        Some(name) => match SubjectRepo::new(&db)
            .list()
            .unwrap_or_default()
            .into_iter()
            .find(|subject| &subject.name == name)
        {
            Some(subject) => Some(subject.id),
            None => {
                eprintln!("предмета «{name}» в базе нет");
                return ExitCode::FAILURE;
            }
        },
        None => None,
    };

    let existing = decks::list(&db)
        .unwrap_or_default()
        .into_iter()
        .find(|deck| deck.name == deck_name);

    let deck = match existing {
        Some(deck) => {
            println!("колода «{}» уже есть, дописываем в неё", deck.name);
            deck
        }
        None => match decks::create(
            &db,
            DeckInput {
                subject_id,
                name: deck_name.clone(),
                description: report.preview.suggested_description.clone(),
            },
        ) {
            Ok(deck) => deck,
            Err(err) => {
                eprintln!("колода не создалась: {}", err.message);
                return ExitCode::FAILURE;
            }
        },
    };

    match import::commit(&db, &deck.id, &report.preview.cards) {
        Ok(written) => {
            println!("записано карточек: {written} в колоду «{}»", deck.name);
            ExitCode::SUCCESS
        }
        Err(err) => {
            eprintln!("импорт не прошёл: {}", err.message);
            ExitCode::FAILURE
        }
    }
}
