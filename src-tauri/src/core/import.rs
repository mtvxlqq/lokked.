//! Bulk import and export of cards.
//!
//! Two formats arrive here. The plain-text one is what a student pastes in:
//! cards divided by one separator, sides by another, both configurable
//! because someone's own notes may already use `---` for something else. The
//! JSON one is a set of lecture cards prepared elsewhere — a title, a
//! statement and the metadata that becomes tags.
//!
//! Both produce the same [`ImportPreview`], which is what the import screen
//! shows before anything is written: the cards it understood, and a list of
//! the blocks it did not. Nothing is dropped silently except genuinely empty
//! blocks — those are just extra separators, not lost cards.
//!
//! Parsing is pure: no clock, no database, no I/O. Behavioural tests live in
//! `src-tauri/tests/import.rs`.

use std::fmt;

use serde::{Deserialize, Serialize};

/// The default separators, and what the import screen offers first.
pub const DEFAULT_CARD_SEPARATOR: &str = "===";
pub const DEFAULT_SIDE_SEPARATOR: &str = "---";

/// One card as it was understood from the text.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ParsedCard {
    pub front: String,
    pub back: String,
    pub hint: Option<String>,
    pub tags: Vec<String>,
}

/// Why one block of the text did not become a card.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "kind")]
pub enum ImportProblemKind {
    /// Only one section: there is a front but nothing to turn over to.
    MissingBack,
    /// A section was there but held nothing but whitespace.
    BlankSide,
    /// More sections than a card has sides.
    TooManySides { found: usize },
}

impl fmt::Display for ImportProblemKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingBack => write!(f, "нет оборотной стороны"),
            Self::BlankSide => write!(f, "одна из сторон пустая"),
            Self::TooManySides { found } => {
                write!(f, "частей {found}, а у карточки их не больше трёх")
            }
        }
    }
}

/// A block that could not be imported, numbered as the student sees it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImportProblem {
    /// 1-based number of the block in the input.
    pub block: usize,
    pub kind: ImportProblemKind,
}

/// What the import screen shows before anything is written.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImportPreview {
    pub cards: Vec<ParsedCard>,
    pub problems: Vec<ImportProblem>,
    /// A deck name the format itself suggests, if it carries one.
    pub suggested_deck: Option<String>,
    pub suggested_description: Option<String>,
}

/// Why an import could not even be started.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ImportError {
    /// A separator was blank, or both were the same.
    UnusableSeparators,
    /// The JSON did not parse, or was not the shape this importer knows.
    NotLectureJson(String),
}

impl fmt::Display for ImportError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnusableSeparators => write!(
                f,
                "разделители не должны быть пустыми и должны отличаться друг от друга"
            ),
            Self::NotLectureJson(why) => write!(f, "файл не похож на набор карточек: {why}"),
        }
    }
}

impl std::error::Error for ImportError {}

/// How the text is cut into cards.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImportOptions {
    pub card_separator: String,
    pub side_separator: String,
}

impl Default for ImportOptions {
    fn default() -> Self {
        Self {
            card_separator: DEFAULT_CARD_SEPARATOR.to_string(),
            side_separator: DEFAULT_SIDE_SEPARATOR.to_string(),
        }
    }
}

impl ImportOptions {
    /// Checks a pair of separators before anything is parsed with them.
    pub fn new(card_separator: &str, side_separator: &str) -> Result<Self, ImportError> {
        let card = card_separator.trim();
        let side = side_separator.trim();

        if card.is_empty() || side.is_empty() || card == side {
            return Err(ImportError::UnusableSeparators);
        }

        Ok(Self {
            card_separator: card.to_string(),
            side_separator: side.to_string(),
        })
    }
}

/// Splits `text` on lines that are nothing but `separator`.
///
/// A line has to *be* the separator, not merely contain it: `5 === 5` is a
/// line of a card, and treating it as a divider would make mathematics
/// impossible to import.
fn split_on_separator_lines<'a>(text: &'a str, separator: &str) -> Vec<&'a str> {
    let mut parts = Vec::new();
    let mut start = 0;
    let mut cursor = 0;

    for line in text.split_inclusive('\n') {
        let is_separator = line.trim_end_matches(['\n', '\r']).trim() == separator;

        if is_separator {
            parts.push(&text[start..cursor]);
            start = cursor + line.len();
        }
        cursor += line.len();
    }
    parts.push(&text[start..]);

    parts
}

/// Parses the plain-text format into a preview.
pub fn parse_text(text: &str, options: &ImportOptions) -> ImportPreview {
    let mut preview = ImportPreview::default();

    for (index, block) in split_on_separator_lines(text, &options.card_separator)
        .into_iter()
        .enumerate()
    {
        // Пустой блок — это два разделителя подряд или разделитель по краям
        // текста. Ошибкой это не является: карточки там и не было.
        if block.trim().is_empty() {
            continue;
        }

        let block_number = index + 1;
        let sides = split_on_separator_lines(block, &options.side_separator);
        let trimmed: Vec<&str> = sides.iter().map(|side| side.trim()).collect();

        match trimmed.as_slice() {
            [_] => preview.problems.push(ImportProblem {
                block: block_number,
                kind: ImportProblemKind::MissingBack,
            }),
            [front, back] | [front, back, _] if front.is_empty() || back.is_empty() => {
                preview.problems.push(ImportProblem {
                    block: block_number,
                    kind: ImportProblemKind::BlankSide,
                })
            }
            [front, back] => preview.cards.push(ParsedCard {
                front: (*front).to_string(),
                back: (*back).to_string(),
                hint: None,
                tags: Vec::new(),
            }),
            [front, back, hint] => preview.cards.push(ParsedCard {
                front: (*front).to_string(),
                back: (*back).to_string(),
                hint: (!hint.is_empty()).then(|| (*hint).to_string()),
                tags: Vec::new(),
            }),
            sides => preview.problems.push(ImportProblem {
                block: block_number,
                kind: ImportProblemKind::TooManySides { found: sides.len() },
            }),
        }
    }

    preview
}

/// Writes cards back out in the same format they are imported from.
pub fn to_text(cards: &[ParsedCard], options: &ImportOptions) -> String {
    let blocks: Vec<String> = cards
        .iter()
        .map(|card| {
            let mut sides = vec![card.front.clone(), card.back.clone()];
            if let Some(hint) = &card.hint {
                sides.push(hint.clone());
            }
            sides.join(&format!("\n{}\n", options.side_separator))
        })
        .collect();

    blocks.join(&format!("\n{}\n", options.card_separator))
}

/// The JSON shape a prepared set of lecture cards arrives in.
#[derive(Debug, Deserialize)]
struct LectureFile {
    meta: Option<LectureMeta>,
    cards: Vec<LectureCard>,
}

#[derive(Debug, Deserialize)]
struct LectureMeta {
    title: Option<String>,
    source: Option<String>,
    scope: Option<String>,
}

#[derive(Debug, Deserialize)]
struct LectureCard {
    /// Название утверждения — лицевая сторона.
    title: Option<String>,
    /// Формулировка — оборотная сторона.
    statement: Option<String>,
    /// «определение», «теорема», … — становится тегом.
    #[serde(rename = "type")]
    kind: Option<String>,
    topic: Option<String>,
    lecture: Option<u32>,
}

/// A parsed lecture file: the cards plus what the deck should be called.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct LectureImport {
    pub preview: ImportPreview,
}

/// Parses the lecture-card JSON.
///
/// The metadata that describes each statement — its type, its topic and the
/// lecture it came from — becomes tags, which is what makes the deck
/// filterable later («покажи только определения из лекции 33»). Everything
/// else in the file is about the source rather than about the card, and is
/// left behind.
pub fn parse_lecture_json(raw: &str) -> Result<LectureImport, ImportError> {
    let file: LectureFile =
        serde_json::from_str(raw).map_err(|err| ImportError::NotLectureJson(err.to_string()))?;

    let mut preview = ImportPreview::default();

    for (index, card) in file.cards.iter().enumerate() {
        let block = index + 1;
        let front = card.title.as_deref().unwrap_or_default().trim();
        let back = card.statement.as_deref().unwrap_or_default().trim();

        if back.is_empty() {
            preview.problems.push(ImportProblem {
                block,
                kind: ImportProblemKind::MissingBack,
            });
            continue;
        }
        if front.is_empty() {
            preview.problems.push(ImportProblem {
                block,
                kind: ImportProblemKind::BlankSide,
            });
            continue;
        }

        let mut tags = Vec::new();
        if let Some(kind) = card
            .kind
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty())
        {
            tags.push(kind.to_string());
        }
        if let Some(topic) = card
            .topic
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty())
        {
            tags.push(topic.to_string());
        }
        if let Some(lecture) = card.lecture {
            tags.push(format!("лекция {lecture}"));
        }

        preview.cards.push(ParsedCard {
            front: front.to_string(),
            back: back.to_string(),
            hint: None,
            tags,
        });
    }

    if let Some(meta) = &file.meta {
        preview.suggested_deck = meta.title.as_deref().map(str::trim).map(str::to_string);
        preview.suggested_description = match (&meta.source, &meta.scope) {
            (Some(source), Some(scope)) => Some(format!("{}. {}", source.trim(), scope.trim())),
            (Some(text), None) | (None, Some(text)) => Some(text.trim().to_string()),
            (None, None) => None,
        };
    }

    Ok(LectureImport { preview })
}
