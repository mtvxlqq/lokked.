//! The card side of the statistics screen: how accuracy went day by day,
//! which cards are worth going back to, and what one card's history says.
//!
//! Everything here is derived from `reviews` rows, which are append-only:
//! the numbers are a reading of what happened, never a state of their own.

use std::collections::HashMap;

use serde::Serialize;

use crate::core::review::Grade;
use crate::core::scheduler::{weakest, CardAccuracy};

use super::percent;
use super::time::day_span;

/// How many of the latest answers the card's history shows as a chain of
/// dots. Ten is what fits on a phone without the dots turning into a line.
pub const RECENT_ANSWERS: usize = 10;

/// One day of card answers.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DayAccuracy {
    pub day_key: String,
    pub answered: u32,
    pub correct: u32,
    /// Zero for a day with no answers — a day nobody studied has no accuracy,
    /// and the chart draws nothing there rather than a column at the floor.
    pub accuracy_percent: u32,
}

/// Answers per day over `[from, to]`, days without answers included.
///
/// `rows` is `(day_key, answered, correct)` in any order and with repeats.
pub fn accuracy_by_day(rows: &[(String, u32, u32)], from: &str, to: &str) -> Vec<DayAccuracy> {
    let mut by_day: HashMap<&str, (u32, u32)> = HashMap::new();
    for (day_key, answered, correct) in rows {
        let entry = by_day.entry(day_key.as_str()).or_insert((0, 0));
        entry.0 += answered;
        entry.1 += correct;
    }

    day_span(from, to)
        .into_iter()
        .map(|date| {
            let day_key = date.format("%Y-%m-%d").to_string();
            let (answered, correct) = by_day.get(day_key.as_str()).copied().unwrap_or((0, 0));

            DayAccuracy {
                day_key,
                answered,
                correct,
                accuracy_percent: percent(correct, answered),
            }
        })
        .collect()
}

/// A card that keeps being missed.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ProblemCard {
    pub card_id: String,
    pub shown: u32,
    pub correct: u32,
    pub accuracy_percent: u32,
}

/// The `limit` weakest cards, worst first.
///
/// The order is [`weakest`]'s, so the list and the «слабые» run agree on
/// what a weak card is — there is one definition of that in the app, and it
/// lives next to the code that deals the cards.
pub fn problem_cards(stats: &[CardAccuracy], limit: usize, min_shows: u32) -> Vec<ProblemCard> {
    let by_id: HashMap<&str, &CardAccuracy> = stats
        .iter()
        .map(|card| (card.card_id.as_str(), card))
        .collect();

    weakest(stats, limit, min_shows)
        .into_iter()
        .filter_map(|card_id| {
            let card = by_id.get(card_id.as_str())?;

            Some(ProblemCard {
                card_id,
                shown: card.shown,
                correct: card.correct,
                accuracy_percent: percent(card.correct, card.shown),
            })
        })
        .collect()
}

/// One answer to one card, as `reviews` recorded it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CardAnswer {
    pub grade: Grade,
    /// Time to the answer being revealed. `None` for a run that did not
    /// measure it.
    pub think_ms: Option<i64>,
}

/// Everything the «по карточке» tab shows.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct CardStats {
    pub shown: u32,
    pub correct: u32,
    pub accuracy_percent: u32,
    /// The last [`RECENT_ANSWERS`] answers, oldest of them first.
    pub recent: Vec<Grade>,
    /// Mean time to recall, rounded; `None` if nothing was ever timed.
    pub average_think_ms: Option<i64>,
    /// Correct answers in a row, counting back from the last one.
    pub current_streak: u32,
}

/// Reduces one card's history, oldest answer first.
pub fn card_stats(answers: &[CardAnswer]) -> CardStats {
    let shown = answers.len() as u32;
    let correct = answers
        .iter()
        .filter(|answer| answer.grade.is_correct())
        .count() as u32;

    let timed: Vec<i64> = answers
        .iter()
        .filter_map(|answer| answer.think_ms)
        .collect();
    let average_think_ms = if timed.is_empty() {
        None
    } else {
        let total: i64 = timed.iter().sum();
        Some((total + timed.len() as i64 / 2) / timed.len() as i64)
    };

    CardStats {
        shown,
        correct,
        accuracy_percent: percent(correct, shown),
        recent: answers
            .iter()
            .rev()
            .take(RECENT_ANSWERS)
            .map(|answer| answer.grade)
            .rev()
            .collect(),
        average_think_ms,
        current_streak: answers
            .iter()
            .rev()
            .take_while(|answer| answer.grade.is_correct())
            .count() as u32,
    }
}
