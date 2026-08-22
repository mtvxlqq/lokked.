//! Choosing what to study and in which order.
//!
//! The five modes differ in exactly two things: how many cards they deal and
//! which ones. Both live here as pure functions.
//!
//! Which card comes next inside a run is not a plain shuffle: every card
//! carries a weight computed from its own history ([`weights`]), and the next
//! one is drawn in proportion to those weights ([`pick`]). Nothing is ever
//! taken out of rotation — a card answered perfectly just comes round
//! rarely.
//!
//! Randomness is seeded rather than taken from the system, so a run can be
//! replayed exactly in a test. The generator is a few lines of arithmetic
//! instead of a dependency — nothing here needs cryptographic quality, only
//! «different every time and the same for the same seed».

use std::fmt;

use serde::{Deserialize, Serialize};

pub mod pick;
pub mod weights;

/// How many cards a classic run deals: one sitting, not one deck.
///
/// A deck of lecture cards is a hundred and more; going through all of it is
/// what the marathon is for.
pub const CLASSIC_LIMIT: usize = 20;

/// How many cards a blitz deals. Same sitting, against a clock.
pub const BLITZ_LIMIT: usize = 20;

/// How many of the weakest cards «слабые» deals.
pub const WEAK_LIMIT: usize = 20;

/// How many cards the reel deals. Same sitting as the classic run — the
/// difference is in how the next card arrives, not in how many there are.
pub const REEL_LIMIT: usize = 20;

/// How many times a card has to have been seen before its accuracy means
/// anything. Below this it is not a weak card, it is a new one.
pub const WEAK_MIN_SHOWS: u32 = 3;

/// How a deck is run through.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StudyMode {
    /// Пачка карточек в случайном порядке.
    Classic,
    /// То же, но на время и со счётом.
    Blitz,
    /// Вся колода за один заход.
    Marathon,
    /// Двадцать карточек с худшей точностью.
    Weak,
    /// Барабан: карточка не показывается, а выпадает.
    Reel,
}

/// A mode the study screen asked for that this module does not know.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnknownMode(pub String);

impl fmt::Display for UnknownMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "неизвестный режим: {}", self.0)
    }
}

impl std::error::Error for UnknownMode {}

impl StudyMode {
    /// The slug stored in `reviews.mode`.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Classic => "classic",
            Self::Blitz => "blitz",
            Self::Marathon => "marathon",
            Self::Weak => "weak",
            Self::Reel => "reel",
        }
    }

    pub fn parse(raw: &str) -> Result<Self, UnknownMode> {
        match raw {
            "classic" => Ok(Self::Classic),
            "blitz" => Ok(Self::Blitz),
            "marathon" => Ok(Self::Marathon),
            "weak" => Ok(Self::Weak),
            "reel" => Ok(Self::Reel),
            other => Err(UnknownMode(other.to_string())),
        }
    }

    /// How many cards to deal, or `None` for «сколько есть».
    pub fn limit(self) -> Option<usize> {
        match self {
            Self::Classic => Some(CLASSIC_LIMIT),
            Self::Blitz => Some(BLITZ_LIMIT),
            Self::Weak => Some(WEAK_LIMIT),
            Self::Reel => Some(REEL_LIMIT),
            Self::Marathon => None,
        }
    }

    /// Whether a card has a deadline of its own.
    pub fn is_timed(self) -> bool {
        matches!(self, Self::Blitz)
    }
}

/// How one card has been going lately.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CardAccuracy {
    pub card_id: String,
    /// How many times it was answered in the window being looked at.
    pub shown: u32,
    /// How many of those were recalled.
    pub correct: u32,
}

/// The cards worth going back to, worst first.
///
/// A card has to have been seen `min_shows` times before its accuracy counts
/// for anything — one miss out of one showing is a first meeting, not a weak
/// spot. Cards with the same accuracy are ordered by how often they were
/// seen, because the one with more showings has the better-evidenced
/// weakness, and then by id, so the same input always gives the same list.
pub fn weakest(stats: &[CardAccuracy], limit: usize, min_shows: u32) -> Vec<String> {
    let mut judged: Vec<&CardAccuracy> = stats
        .iter()
        .filter(|card| card.shown >= min_shows)
        .collect();

    judged.sort_by(|left, right| {
        // Сравнение долей без плавающей точки: correct/shown < correct/shown.
        let left_accuracy = left.correct as u64 * right.shown as u64;
        let right_accuracy = right.correct as u64 * left.shown as u64;

        left_accuracy
            .cmp(&right_accuracy)
            .then(right.shown.cmp(&left.shown))
            .then(left.card_id.cmp(&right.card_id))
    });

    judged
        .into_iter()
        .take(limit)
        .map(|card| card.card_id.clone())
        .collect()
}

/// A seeded pseudo-random generator (SplitMix64).
///
/// Small, fast, and good enough for shuffling a deck: it passes the usual
/// statistical batteries, and it has no state to speak of beyond one number.
pub struct Rng {
    state: u64,
}

impl Rng {
    pub fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    pub fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// A number in `[0, 1)`, drawn from the top 53 bits — the ones a `f64`
    /// can hold without rounding.
    pub fn next_f64(&mut self) -> f64 {
        (self.next_u64() >> 11) as f64 / (1_u64 << 53) as f64
    }

    /// A number in `[0, bound)`, without the bias a plain `%` would add.
    ///
    /// Values from the last, incomplete stretch of the 64-bit range are
    /// thrown away and redrawn; with a bound as small as a deck size that
    /// practically never happens even once.
    pub fn below(&mut self, bound: u64) -> u64 {
        if bound <= 1 {
            return 0;
        }

        let limit = u64::MAX - (u64::MAX % bound) - 1;
        loop {
            let value = self.next_u64();
            if value <= limit {
                return value % bound;
            }
        }
    }
}

/// Shuffles in place, Fisher–Yates, using `seed`.
///
/// Every ordering is equally likely, and the same seed always gives the same
/// one — which is what makes a run reproducible in a test.
pub fn shuffle<T>(items: &mut [T], seed: u64) {
    let mut rng = Rng::new(seed);

    for index in (1..items.len()).rev() {
        let swap_with = rng.below(index as u64 + 1) as usize;
        items.swap(index, swap_with);
    }
}
