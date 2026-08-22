//! Drawing the next card out of a deck of weights.
//!
//! Two ways of using the same weights, one per kind of run:
//!
//! - [`Picker`] deals card after card, with replacement — the run has a
//!   length of its own and a card may come round more than once inside it.
//!   The last few cards dealt are held back, so nothing repeats immediately
//!   however heavy it is.
//! - [`weighted_order`] lays the whole deck out in one order, without
//!   replacement — what a marathon needs, where every card comes exactly
//!   once and the weight decides how early.
//!
//! Both take a seed and take it from the caller, so a run can be replayed
//! exactly in a test.

use std::collections::VecDeque;

use super::Rng;

/// How many of the most recently dealt cards are held back from the draw.
///
/// Three is enough to break up the repetition a high weight would otherwise
/// cause, and short enough that a card just missed still comes back inside
/// the next handful.
pub const REPEAT_WINDOW: usize = 3;

/// A card and how much it wants to come round.
#[derive(Debug, Clone, PartialEq)]
pub struct Weighted {
    pub card_id: String,
    pub weight: f64,
}

/// Deals cards at random, in proportion to their weights.
///
/// The weights are not fixed for the life of the picker: [`reweigh`] updates
/// one as a run goes, which is how a card answered «не помню» climbs back to
/// the front.
///
/// [`reweigh`]: Picker::reweigh
pub struct Picker {
    cards: Vec<Weighted>,
    /// Indices of the last few cards dealt, oldest first.
    recent: VecDeque<usize>,
    window: usize,
    rng: Rng,
}

impl Picker {
    /// `window` is how many of the last cards to hold back. It is capped at
    /// half the deck: a deck of one card has nothing else to deal, and on a
    /// deck of four a window of three would leave exactly one card to
    /// «choose» from, turning the run into a fixed cycle.
    pub fn new(cards: Vec<Weighted>, window: usize, seed: u64) -> Self {
        let window = window.min(cards.len() / 2);

        Self {
            cards,
            recent: VecDeque::new(),
            window,
            rng: Rng::new(seed),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.cards.is_empty()
    }

    pub fn len(&self) -> usize {
        self.cards.len()
    }

    /// The next card, or `None` if there is no deck to deal from.
    pub fn pick(&mut self) -> Option<String> {
        if self.cards.is_empty() {
            return None;
        }

        let available: Vec<usize> = (0..self.cards.len())
            .filter(|index| !self.recent.contains(index))
            .collect();
        // Окно уже сужено до `len - 1`, так что что-то доступное есть всегда;
        // пустой список означал бы ошибку в сужении, а не пустую колоду.
        let chosen = self.roll(&available).unwrap_or(0);

        self.recent.push_back(chosen);
        while self.recent.len() > self.window {
            self.recent.pop_front();
        }

        Some(self.cards[chosen].card_id.clone())
    }

    /// Picks one of `available` in proportion to its weight.
    fn roll(&mut self, available: &[usize]) -> Option<usize> {
        let total: f64 = available
            .iter()
            .map(|index| self.cards[*index].weight.max(0.0))
            .sum();

        if total <= 0.0 {
            // Вес не бывает нулевым, но если все до одного схлопнулись,
            // прогон должен продолжиться, а не остановиться.
            return available.first().copied();
        }

        let mut roll = self.rng.next_f64() * total;
        for index in available {
            roll -= self.cards[*index].weight.max(0.0);
            if roll < 0.0 {
                return Some(*index);
            }
        }

        available.last().copied()
    }

    /// Sets a card's weight to what its history now says it is.
    ///
    /// A card the deck does not hold is ignored: a run only ever reweighs
    /// what it just dealt.
    pub fn reweigh(&mut self, card_id: &str, weight: f64) {
        if let Some(card) = self.cards.iter_mut().find(|card| card.card_id == card_id) {
            card.weight = weight;
        }
    }
}

/// Lays the whole deck out in one order, heavy cards tending to the front.
///
/// Each place is drawn from what is left, in proportion to weight, so a heavy
/// card is likely to come early but is never guaranteed to — the marathon
/// stays a run through the deck, not a ranking of it.
pub fn weighted_order(cards: Vec<Weighted>, seed: u64) -> Vec<String> {
    let mut rest = cards;
    let mut rng = Rng::new(seed);
    let mut order = Vec::with_capacity(rest.len());

    while !rest.is_empty() {
        let total: f64 = rest.iter().map(|card| card.weight.max(0.0)).sum();
        let mut roll = if total > 0.0 {
            rng.next_f64() * total
        } else {
            0.0
        };

        let mut chosen = rest.len() - 1;
        for (index, card) in rest.iter().enumerate() {
            roll -= card.weight.max(0.0);
            if roll < 0.0 {
                chosen = index;
                break;
            }
        }

        order.push(rest.swap_remove(chosen).card_id);
    }

    order
}
