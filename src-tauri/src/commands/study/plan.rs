//! Which card a run deals next, and what it learns from the answer.
//!
//! Two shapes of run, one type. A marathon, «слабые» and a repeat of the
//! mistakes know their whole queue before they start — the order is decided
//! once and followed. A classic run, a blitz and the reel deal a card at a
//! time out of the deck, drawn in proportion to how much each card wants to
//! come round; that weight is recomputed after every answer, which is what
//! brings a card just missed back within the next few.
//!
//! The histories are kept here for the length of the run rather than being
//! read back out of the database per card: a run answers twenty cards, and
//! [`crate::core::scheduler::weights`] needs only the last ten answers of
//! each to say what a card is worth.

use std::collections::HashMap;

use chrono::{DateTime, Utc};

use crate::core::review::Grade;
use crate::core::scheduler::pick::{weighted_order, Picker, Weighted, REPEAT_WINDOW};
use crate::core::scheduler::weights::{weight, Answer, CardHistory};
use crate::db::cards::WeightCache;

use super::super::cards::CardView;

/// How a run picks its cards, and what it knows about them.
pub(crate) struct Plan {
    /// The cards the run may deal.
    pub(crate) pool: Vec<CardView>,
    /// What `reviews` remembers about each of them, brought up to date as
    /// the run goes.
    histories: HashMap<String, CardHistory>,
    /// How sharply the weights lean towards the weak cards.
    exponent: f64,
    /// `None` when the queue was decided up front.
    picker: Option<Picker>,
}

impl Plan {
    /// A run whose order is already settled.
    pub(crate) fn fixed(
        queue: Vec<CardView>,
        histories: HashMap<String, CardHistory>,
        exponent: f64,
    ) -> Self {
        Self {
            pool: queue,
            histories,
            exponent,
            picker: None,
        }
    }

    /// A run that draws its next card as it goes.
    pub(crate) fn adaptive(
        pool: Vec<CardView>,
        histories: HashMap<String, CardHistory>,
        exponent: f64,
        now: DateTime<Utc>,
        seed: u64,
    ) -> Self {
        let picker = Picker::new(weigh(&pool, &histories, exponent, now), REPEAT_WINDOW, seed);

        Self {
            pool,
            histories,
            exponent,
            picker: Some(picker),
        }
    }

    /// A fresh plan over `cards`, in the order given, carrying over what
    /// this run has learned so far.
    ///
    /// What the repeat of a run's mistakes starts from: the answers just
    /// given are part of those cards' history, and starting from a blank one
    /// would write a wrong count into the cache on the next answer.
    pub(crate) fn repeat(&self, cards: Vec<CardView>) -> Self {
        Self {
            pool: cards,
            histories: self.histories.clone(),
            exponent: self.exponent,
            picker: None,
        }
    }

    /// Whether the next card is still to be drawn.
    pub(crate) fn deals_as_it_goes(&self) -> bool {
        self.picker.is_some()
    }

    /// The next card to put on screen, or `None` for a run that already
    /// knows its queue.
    pub(crate) fn deal(&mut self) -> Option<CardView> {
        let picked = self.picker.as_mut()?.pick()?;

        self.pool.iter().find(|card| card.id == picked).cloned()
    }

    /// Files an answer and reweighs the card it was given to.
    ///
    /// Returns what to cache on the card, so the row in `cards` keeps up with
    /// what the picker believes without anyone having to recompute it later.
    pub(crate) fn answered(
        &mut self,
        card_id: &str,
        grade: Grade,
        at: DateTime<Utc>,
    ) -> WeightCache {
        let history = self
            .histories
            .entry(card_id.to_string())
            .or_insert_with(|| CardHistory::new(card_id));
        history.answered(Answer { at, grade });

        let value = weight(history, at, self.exponent);
        let cache = WeightCache {
            weight: value,
            reps: history.shows as i64,
            lapses: history
                .recent
                .iter()
                .filter(|answer| !answer.grade.is_correct())
                .count() as i64,
        };

        if let Some(picker) = self.picker.as_mut() {
            picker.reweigh(card_id, value);
        }

        cache
    }
}

/// Weighs every card of a pool at one moment.
fn weigh(
    pool: &[CardView],
    histories: &HashMap<String, CardHistory>,
    exponent: f64,
    now: DateTime<Utc>,
) -> Vec<Weighted> {
    pool.iter()
        .map(|card| Weighted {
            card_id: card.id.clone(),
            weight: match histories.get(&card.id) {
                Some(history) => weight(history, now, exponent),
                // Карточку ни разу не отвечали — вес у неё средний.
                None => weight(&CardHistory::new(&card.id), now, exponent),
            },
        })
        .collect()
}

/// Lays a whole deck out with the weak cards tending to the front.
///
/// What a marathon runs in: every card comes exactly once, and the weight
/// decides how early rather than whether at all.
pub(crate) fn order_by_weight(
    cards: Vec<CardView>,
    histories: &HashMap<String, CardHistory>,
    exponent: f64,
    now: DateTime<Utc>,
    seed: u64,
) -> Vec<CardView> {
    let order = weighted_order(weigh(&cards, histories, exponent, now), seed);

    order
        .iter()
        .filter_map(|id| cards.iter().find(|card| &card.id == id).cloned())
        .collect()
}

/// Groups the rows of `reviews` into one history per card.
///
/// The rows arrive oldest first, which is the order
/// [`CardHistory::answered`] expects. An answer whose grade the app does not
/// know is skipped rather than guessed at — a row from a newer version has
/// no business changing what a card is worth here.
pub(crate) fn histories_from_rows(
    rows: Vec<(String, DateTime<Utc>, String)>,
) -> HashMap<String, CardHistory> {
    let mut histories: HashMap<String, CardHistory> = HashMap::new();

    for (card_id, reviewed_at, result) in rows {
        let Ok(grade) = Grade::parse(&result) else {
            continue;
        };

        histories
            .entry(card_id.clone())
            .or_insert_with(|| CardHistory::new(&card_id))
            .answered(Answer {
                at: reviewed_at,
                grade,
            });
    }

    histories
}
