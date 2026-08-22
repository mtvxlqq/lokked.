//! The rules of a duel: who plays, what a sitting is made of, who won.
//!
//! A duel is a blitz two to four people take turns at on one device. Everyone
//! answers the same cards in the same order — otherwise the scores compare
//! nothing — and between turns the device is handed over with the previous
//! result hidden.
//!
//! Scoring is the blitz scoring of [`crate::core::stats::run`], unchanged: a
//! duel is not a different game, it is the same game with someone watching.
//!
//! Nothing here reads the clock, the database or the deck. Which cards fall
//! out is decided where the randomness lives — [`crate::core::scheduler`] —
//! and stored answers are the command layer's business.

use std::collections::HashSet;
use std::fmt;

use serde::{Deserialize, Serialize};

use crate::core::review::Grade;
use crate::core::settings::{MAX_BLITZ_SECONDS, MIN_BLITZ_SECONDS};

/// How many people a duel takes.
///
/// Two is the point of it; above four the device spends longer being passed
/// around than being answered on.
pub const MIN_PLAYERS: usize = 2;
pub const MAX_PLAYERS: usize = 4;

/// How many cards one duel runs over.
pub const MIN_DUEL_CARDS: usize = 5;
pub const MAX_DUEL_CARDS: usize = 50;

/// The default sitting: the same twenty cards a blitz deals.
pub const DEFAULT_DUEL_CARDS: usize = 20;

/// Why a duel could not be set up.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DuelError {
    TooFewPlayers(usize),
    TooManyPlayers(usize),
    /// A player was added without a name.
    EmptyName,
    /// Two players called the same thing, so the table would be unreadable.
    DuplicateName(String),
    InvalidCards(usize),
    InvalidSeconds(i64),
    /// The deck has fewer cards than the duel asked for.
    NotEnoughCards {
        asked: usize,
        has: usize,
    },
}

impl fmt::Display for DuelError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TooFewPlayers(count) => {
                write!(f, "дуэль — это хотя бы {MIN_PLAYERS} игрока, а не {count}")
            }
            Self::TooManyPlayers(count) => write!(
                f,
                "на одном устройстве играют максимум {MAX_PLAYERS} игрока, а не {count}"
            ),
            Self::EmptyName => write!(f, "у игрока должно быть имя"),
            Self::DuplicateName(name) => {
                write!(f, "двух игроков с именем «{name}» будет не различить")
            }
            Self::InvalidCards(count) => write!(
                f,
                "в дуэли от {MIN_DUEL_CARDS} до {MAX_DUEL_CARDS} карточек, а не {count}"
            ),
            Self::InvalidSeconds(seconds) => write!(
                f,
                "на карточку нужно от {MIN_BLITZ_SECONDS} до {MAX_BLITZ_SECONDS} секунд, а не {seconds}"
            ),
            Self::NotEnoughCards { asked, has } => write!(
                f,
                "в колоде {has} карточек — на дуэль из {asked} не хватит"
            ),
        }
    }
}

impl std::error::Error for DuelError {}

/// One player of a duel.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DuelPlayer {
    pub name: String,
    /// Whether this is the student whose device it is.
    ///
    /// Only their answers go into `reviews`: a guest's evening should not
    /// show up in someone else's statistics or move their card weights.
    pub is_owner: bool,
}

/// A duel that has been agreed on but not yet dealt.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DuelSetup {
    pub players: Vec<DuelPlayer>,
    pub cards: usize,
    pub seconds_per_card: i64,
}

impl DuelSetup {
    /// Validates what the setup screen collected.
    ///
    /// The first name is the owner of the device — the screen puts «Ты»
    /// there and the rest are guests.
    pub fn new(names: Vec<String>, cards: usize, seconds_per_card: i64) -> Result<Self, DuelError> {
        if names.len() < MIN_PLAYERS {
            return Err(DuelError::TooFewPlayers(names.len()));
        }
        if names.len() > MAX_PLAYERS {
            return Err(DuelError::TooManyPlayers(names.len()));
        }
        if !(MIN_DUEL_CARDS..=MAX_DUEL_CARDS).contains(&cards) {
            return Err(DuelError::InvalidCards(cards));
        }
        if !(MIN_BLITZ_SECONDS..=MAX_BLITZ_SECONDS).contains(&seconds_per_card) {
            return Err(DuelError::InvalidSeconds(seconds_per_card));
        }

        let mut seen: HashSet<String> = HashSet::new();
        let mut players = Vec::with_capacity(names.len());

        for (position, name) in names.iter().enumerate() {
            let name = name.trim();
            if name.is_empty() {
                return Err(DuelError::EmptyName);
            }
            // Сравнение без регистра: «Артём» и «артём» за столом — один и
            // тот же человек, и в таблице их не различить.
            if !seen.insert(name.to_lowercase()) {
                return Err(DuelError::DuplicateName(name.to_lowercase()));
            }

            players.push(DuelPlayer {
                name: name.to_string(),
                is_owner: position == 0,
            });
        }

        Ok(Self {
            players,
            cards,
            seconds_per_card,
        })
    }
}

/// Which players scored the most, by index.
///
/// More than one on a draw, and none at all when nobody scored: a duel where
/// every answer was wrong has no winner to congratulate.
pub fn winners(points: &[i64]) -> Vec<usize> {
    let best = points.iter().copied().max().unwrap_or(0);
    if best <= 0 {
        return Vec::new();
    }

    points
        .iter()
        .enumerate()
        .filter(|(_, scored)| **scored == best)
        .map(|(index, _)| index)
        .collect()
}

/// One card of the duel, and what each player answered to it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CardBreakdown {
    pub card_id: String,
    /// One entry per player, in turn order. `None` where a player never got
    /// that far — a duel can be left halfway.
    pub answers: Vec<Option<Grade>>,
}

/// Lays the answers out as the summary table draws them: a row per card, a
/// column per player.
///
/// `answers` is `(player, position, grade)` in any order; a repeat for the
/// same cell keeps the first one, because a duel writes an answer once.
pub fn breakdown(
    cards: &[String],
    players: usize,
    answers: &[(usize, usize, Grade)],
) -> Vec<CardBreakdown> {
    let mut rows: Vec<CardBreakdown> = cards
        .iter()
        .map(|card_id| CardBreakdown {
            card_id: card_id.clone(),
            answers: vec![None; players],
        })
        .collect();

    for (player, position, grade) in answers {
        let Some(row) = rows.get_mut(*position) else {
            continue;
        };
        let Some(cell) = row.answers.get_mut(*player) else {
            continue;
        };
        cell.get_or_insert(*grade);
    }

    rows
}
