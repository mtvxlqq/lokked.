//! The numbers under one finished run of cards: how it went, and what the
//! blitz scored.
//!
//! Pure reducers over the answers a run collected. They never look at the
//! database or the clock: the run screen holds its answers in memory and
//! hands them over as a slice.

use serde::Serialize;

use crate::core::review::Grade;

use super::percent;

/// One answered card, as a finished run remembers it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReviewOutcome {
    pub card_id: String,
    pub grade: Grade,
    /// Time from the card appearing to the grade being given.
    pub total_ms: i64,
}

/// What the screen after a run shows.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct ReviewSummary {
    pub answered: usize,
    pub correct: usize,
    /// Rounded to the nearest per cent; zero for a run with no answers.
    pub accuracy_percent: u32,
    pub total_ms: i64,
    /// Mean time per card, rounded; zero for a run with no answers.
    pub average_ms: i64,
    /// Cards answered «не помню», in the order they came up. Not deduplicated:
    /// a card answered twice in one run really was missed twice.
    pub mistakes: Vec<String>,
}

/// Turns the answers of one run into the numbers under it.
pub fn review_summary(results: &[ReviewOutcome]) -> ReviewSummary {
    let answered = results.len();
    if answered == 0 {
        return ReviewSummary::default();
    }

    let correct = results
        .iter()
        .filter(|result| result.grade.is_correct())
        .count();
    let total_ms: i64 = results.iter().map(|result| result.total_ms).sum();

    ReviewSummary {
        answered,
        correct,
        accuracy_percent: percent(correct as u32, answered as u32),
        total_ms,
        average_ms: (total_ms + answered as i64 / 2) / answered as i64,
        mistakes: results
            .iter()
            .filter(|result| !result.grade.is_correct())
            .map(|result| result.card_id.clone())
            .collect(),
    }
}

/// Points a correct answer is worth before any multiplier.
pub const BLITZ_POINTS: i64 = 10;

/// Answers in a row that start paying one and a half.
pub const BLITZ_STREAK_HALF: u32 = 5;

/// Answers in a row that start paying double.
pub const BLITZ_STREAK_DOUBLE: u32 = 10;

/// What a blitz run scored.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize)]
pub struct BlitzScore {
    pub points: i64,
    /// The longest run of recalled cards in a row.
    pub best_streak: u32,
}

/// Counts a blitz run.
///
/// Ten points a card, one and a half times that from the fifth in a row and
/// double from the tenth — the multiplier applies to the answer that reaches
/// the streak, not retroactively. A miss is worth nothing and puts the
/// streak back to zero.
pub fn blitz_score(results: &[ReviewOutcome]) -> BlitzScore {
    let mut score = BlitzScore::default();
    let mut streak = 0;

    for result in results {
        if !result.grade.is_correct() {
            streak = 0;
            continue;
        }

        streak += 1;
        score.best_streak = score.best_streak.max(streak);
        score.points += match streak {
            s if s >= BLITZ_STREAK_DOUBLE => BLITZ_POINTS * 2,
            s if s >= BLITZ_STREAK_HALF => BLITZ_POINTS * 3 / 2,
            _ => BLITZ_POINTS,
        };
    }

    score
}
