//! How much a card wants to come round.
//!
//! Every card in a deck carries a weight, and the next card is drawn at
//! random in proportion to those weights (see [`super::pick`]). The weight is
//! a pure function of what `reviews` remembers about the card: how it has
//! been going lately, how long it has been out of sight, what the last answer
//! was, and how much of a history there is to judge by at all.
//!
//! Two properties are load-bearing and are not to be optimised away:
//!
//! 1. **No weight is ever zero.** A card answered perfectly fifty times in a
//!    row settles at a small weight and stays there — it will come round
//!    rarely, but it will come round. Cards never disappear from a deck the
//!    way a due-date queue hides them.
//! 2. **The last answer counts for a lot.** «Не помню» has to bring a card
//!    back within the next few cards, not at the next sitting, which is why
//!    the weight is recomputed as a run goes rather than fixed at its start.
//!
//! The columns `ease`, `interval_days`, `due_at`, `reps` and `lapses` on
//! `cards` are a cache of what this module computes. `reviews` stays the
//! source of truth: any weight here can be recomputed from scratch.

use chrono::{DateTime, Utc};

use crate::core::review::Grade;

/// How many of the most recent answers count towards a card's accuracy.
///
/// Ten is enough to tell a card that is going badly from one bad answer, and
/// short enough that a card fixed a month ago is not still called weak.
pub const RECENT_ANSWERS: usize = 10;

/// How quickly older answers inside that window stop mattering.
///
/// Each step back multiplies an answer's say by this, so the newest answer
/// weighs about four times what the fifth-newest does.
const ANSWER_DECAY: f64 = 0.75;

/// The weight of a card nobody has answered yet.
///
/// Deliberately the middle of the range rather than the top: a new card
/// should join the rotation, not take it over. Everything else is measured
/// against this — a card going badly climbs above it, a card going well
/// sinks below.
pub const NEW_WEIGHT: f64 = 0.5;

/// How many answers it takes for a card's own history to speak for itself
/// when that history is a good one.
///
/// Below this a card going well is still weighed near [`NEW_WEIGHT`]: one
/// lucky answer is not evidence of anything. A card going badly is taken at
/// its word straight away.
const PRIOR_ANSWERS: f64 = 3.0;

/// Where a card answered perfectly settles. Small, never zero.
const MASTERED_WEIGHT: f64 = 0.08;

/// How much being out of sight can add, at most — a card unseen for a long
/// time weighs about twice what the same card seen an hour ago does.
const RECENCY_GAIN: f64 = 1.0;

/// How fast that build-up happens, in days.
const RECENCY_SCALE_DAYS: f64 = 3.0;

/// The smallest weight any card can end up with, whatever the arithmetic.
///
/// Guards requirement 1 against a very high aggressiveness driving a small
/// weight to zero.
pub const MIN_WEIGHT: f64 = 1e-3;

/// What one answer is worth as «recalled», from nothing to fully.
fn recall_score(grade: Grade) -> f64 {
    match grade {
        Grade::Again => 0.0,
        Grade::Hard => 0.5,
        Grade::Good => 0.9,
        Grade::Easy => 1.0,
    }
}

/// How much the last answer alone pulls the weight up or down.
fn last_answer_factor(grade: Grade) -> f64 {
    match grade {
        Grade::Again => 4.0,
        Grade::Hard => 1.6,
        Grade::Good => 1.0,
        Grade::Easy => 0.8,
    }
}

/// One answer, as far as the weight is concerned.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Answer {
    pub at: DateTime<Utc>,
    pub grade: Grade,
}

/// What is known about one card: how often it has been answered, and how the
/// last few of those answers went.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CardHistory {
    pub card_id: String,
    /// Every answer ever given, including the ones past the window.
    pub shows: u32,
    /// The last [`RECENT_ANSWERS`] answers, oldest first.
    pub recent: Vec<Answer>,
}

impl CardHistory {
    pub fn new(card_id: impl Into<String>) -> Self {
        Self {
            card_id: card_id.into(),
            shows: 0,
            recent: Vec::new(),
        }
    }

    /// Files one more answer, dropping whatever falls out of the window.
    ///
    /// Answers are expected in the order they were given, which is the order
    /// both the database and a run produce them in.
    pub fn answered(&mut self, answer: Answer) {
        self.shows += 1;
        self.recent.push(answer);

        if self.recent.len() > RECENT_ANSWERS {
            self.recent.remove(0);
        }
    }

    /// Accuracy over the window, with the newest answers counting for more.
    fn accuracy(&self) -> Option<f64> {
        let mut total = 0.0;
        let mut scored = 0.0;

        for (steps_back, answer) in self.recent.iter().rev().enumerate() {
            let say = ANSWER_DECAY.powi(steps_back as i32);
            total += say;
            scored += say * recall_score(answer.grade);
        }

        (total > 0.0).then(|| scored / total)
    }
}

/// How much this card wants to come round, given what is known about it.
///
/// `exponent` is the aggressiveness of the selection, as
/// [`crate::core::settings::AdaptiveSettings::exponent`] computes it: `0`
/// flattens every weight to `1` — a plain shuffle — and larger values stretch
/// the gap between a weak card and a known one.
///
/// The result is a bare number, not a probability: only the ratios between
/// the weights of one deck mean anything.
pub fn weight(card: &CardHistory, now: DateTime<Utc>, exponent: f64) -> f64 {
    let Some(accuracy) = card.accuracy() else {
        // Ничего не отвечали — судить не по чему, вес средний.
        return NEW_WEIGHT.powf(exponent).max(MIN_WEIGHT);
    };

    let mastery = MASTERED_WEIGHT + (1.0 - MASTERED_WEIGHT) * (1.0 - accuracy);
    let last = card.recent.last().expect("окно не пусто");
    let raw = mastery * recency(last.at, now) * last_answer_factor(last.grade);

    // Пока показов мало, вес тянется к среднему: две правильные подряд ещё
    // не значат, что карточка выучена. Вверх это не работает — одного «не
    // помню» достаточно, чтобы показать карточку снова, и ждать статистики
    // тут нечего.
    let blended = if raw > NEW_WEIGHT {
        raw
    } else {
        let confidence = card.shows as f64 / (card.shows as f64 + PRIOR_ANSWERS);
        NEW_WEIGHT + (raw - NEW_WEIGHT) * confidence
    };

    blended.max(MIN_WEIGHT).powf(exponent).max(MIN_WEIGHT)
}

/// How much being out of sight adds, from `1` just after an answer up to
/// `1 + RECENCY_GAIN` for a card nobody has seen in weeks.
fn recency(last_seen: DateTime<Utc>, now: DateTime<Utc>) -> f64 {
    let days = (now - last_seen).num_seconds().max(0) as f64 / 86_400.0;

    1.0 + RECENCY_GAIN * (1.0 - (-days / RECENCY_SCALE_DAYS).exp())
}
