//! A run through a deck: which card is on screen, what was answered, and
//! what the run added up to.
//!
//! The run lives in Tauri's managed state rather than in the frontend, for
//! the same reason the timer does: which card comes next, how long the
//! student looked at it and whether that counts as recalled are decisions,
//! not decoration. The screen shows what it is given.
//!
//! Timing is measured here too, from timestamps: `think_ms` is the card
//! appearing to the answer being revealed, `total_ms` the card appearing to
//! the grade, and a blitz deadline is `shown_at + seconds`. Nothing is
//! accumulated by ticking, so a backgrounded window cannot gain a student
//! time they did not have.
//!
//! This module holds the state and the shapes; [`actions`] holds what can be
//! done to it.

use std::sync::Mutex;

use chrono::{DateTime, TimeDelta, Utc};
use serde::Serialize;

use crate::core::clock::Clock;
use crate::core::scheduler::StudyMode;
use crate::core::stats::{blitz_score, review_summary, ReviewOutcome, ReviewSummary};

use super::cards::CardView;
use super::{CommandError, ErrorKind};

pub mod actions;
pub(crate) mod plan;

use plan::Plan;

/// One card in the queue, with the answer kept back until it is revealed.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct StudyCardView {
    pub id: String,
    pub front: String,
    /// `null` until the student has asked to see it.
    pub back: Option<String>,
    pub hint: Option<String>,
    pub tags: Vec<String>,
}

/// What the review screen draws.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct StudyView {
    pub deck_id: String,
    pub deck_name: String,
    /// `'classic' | 'blitz' | 'marathon' | 'weak'`.
    pub mode: String,
    pub total: usize,
    /// 1-based number of the card on screen; equals `total` on the last one.
    pub position: usize,
    pub answered: usize,
    pub revealed: bool,
    /// `null` once the run is over — the screen switches to the summary.
    pub card: Option<StudyCardView>,
    pub finished: bool,
    /// When the card on screen runs out of time. Blitz only.
    pub deadline: Option<DateTime<Utc>>,
    /// How long a card gets in total, so the ring knows what a full turn is.
    pub seconds_per_card: Option<i64>,
    /// Points so far. Blitz only.
    pub points: Option<i64>,
    /// Cards recalled in a row right now — what the multiplier rides on.
    pub streak: Option<u32>,
}

/// The summary screen: the numbers, plus the cards that were missed.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct StudySummaryView {
    pub deck_id: String,
    pub deck_name: String,
    pub mode: String,
    #[serde(flatten)]
    pub summary: ReviewSummary,
    /// The missed cards themselves, so the screen can list them by name.
    pub mistake_cards: Vec<StudyCardView>,
    /// Blitz only: what this run scored, the deck's record, and whether the
    /// run is the one that set it.
    pub points: Option<i64>,
    pub best_streak: Option<u32>,
    pub record: Option<i64>,
    pub record_beaten: bool,
}

/// The run in progress, or the one that has just finished.
pub struct StudyRun {
    pub(crate) deck_id: String,
    pub(crate) deck_name: String,
    pub(crate) mode: StudyMode,
    /// The cards in the order they were dealt, up to and including the one
    /// on screen. A run that draws as it goes fills this in one card at a
    /// time, so a card may appear in it more than once.
    pub(crate) queue: Vec<CardView>,
    /// How many cards the run deals in total.
    pub(crate) total: usize,
    /// Where the next card comes from, and what is known about the deck.
    pub(crate) plan: Plan,
    /// Index of the card on screen; equals [`total`](Self::total) when the
    /// run is over.
    pub(crate) position: usize,
    pub(crate) shown_at: DateTime<Utc>,
    pub(crate) revealed_at: Option<DateTime<Utc>>,
    pub(crate) results: Vec<ReviewOutcome>,
    /// How long each card lasts, in a timed mode.
    pub(crate) seconds_per_card: Option<i64>,
    /// Set when a blitz run finished and beat what was stored.
    pub(crate) record_beaten: bool,
}

impl StudyRun {
    /// When the card on screen runs out of time, if it can.
    pub(crate) fn deadline(&self) -> Option<DateTime<Utc>> {
        self.seconds_per_card
            .map(|seconds| self.shown_at + TimeDelta::seconds(seconds))
    }

    /// Whether the card on screen has already run out of time.
    pub(crate) fn is_late(&self, now: DateTime<Utc>) -> bool {
        self.deadline().is_some_and(|deadline| now > deadline)
    }

    /// Cards recalled in a row at this moment.
    pub(crate) fn streak(&self) -> u32 {
        self.results
            .iter()
            .rev()
            .take_while(|result| result.grade.is_correct())
            .count() as u32
    }
}

/// Managed state: the run, or nothing.
#[derive(Default)]
pub struct StudyState(Mutex<Option<StudyRun>>);

impl StudyState {
    pub(crate) fn lock(&self) -> std::sync::MutexGuard<'_, Option<StudyRun>> {
        self.0.lock().expect("study mutex poisoned")
    }
}

pub(crate) fn no_run() -> CommandError {
    CommandError {
        kind: ErrorKind::Conflict,
        message: "прогон не идёт".to_string(),
    }
}

pub(crate) fn conflict(message: &str) -> CommandError {
    CommandError {
        kind: ErrorKind::Conflict,
        message: message.to_string(),
    }
}

pub(crate) fn card_view(card: &CardView, revealed: bool) -> StudyCardView {
    StudyCardView {
        id: card.id.clone(),
        front: card.front.clone(),
        // Оборот не уезжает на экран до раскрытия: иначе «время до ответа»
        // измеряло бы не то, а подсмотреть было бы нечем помешать.
        back: revealed.then(|| card.back.clone()),
        hint: card.hint.clone(),
        tags: card.tags.clone(),
    }
}

pub(crate) fn view(run: &StudyRun) -> StudyView {
    let finished = run.position >= run.total;
    let revealed = run.revealed_at.is_some();
    let timed = run.mode.is_timed();

    StudyView {
        deck_id: run.deck_id.clone(),
        deck_name: run.deck_name.clone(),
        mode: run.mode.as_str().to_string(),
        total: run.total,
        position: (run.position + 1).min(run.total),
        answered: run.results.len(),
        revealed,
        card: run
            .queue
            .get(run.position)
            .map(|card| card_view(card, revealed)),
        finished,
        deadline: (!finished).then(|| run.deadline()).flatten(),
        seconds_per_card: run.seconds_per_card,
        points: timed.then(|| blitz_score(&run.results).points),
        streak: timed.then(|| run.streak()),
    }
}

/// Builds a run out of the cards a [`Plan`] holds.
///
/// The length is the mode's sitting, never more than the deck itself: the
/// whole deck for a marathon, twenty cards for everything else. A run that
/// draws as it goes starts with one card and asks the plan for the next one
/// after every answer; a run whose order was settled up front simply follows
/// it.
pub(crate) fn begin(
    deck_id: &str,
    deck_name: &str,
    mode: StudyMode,
    mut plan: Plan,
    seconds_per_card: Option<i64>,
    clock: &dyn Clock,
) -> Result<StudyRun, CommandError> {
    if plan.pool.is_empty() {
        return Err(conflict("в колоде нет карточек"));
    }

    let total = mode.limit().unwrap_or(plan.pool.len()).min(plan.pool.len());
    let queue = if plan.deals_as_it_goes() {
        plan.deal().into_iter().collect()
    } else {
        plan.pool.iter().take(total).cloned().collect()
    };

    Ok(StudyRun {
        deck_id: deck_id.to_string(),
        deck_name: deck_name.to_string(),
        mode,
        queue,
        total,
        plan,
        position: 0,
        shown_at: clock.now(),
        revealed_at: None,
        results: Vec::new(),
        seconds_per_card,
        record_beaten: false,
    })
}

/// The numbers under a run, with the blitz score when there is one.
pub(crate) fn summarise(run: &StudyRun, record: Option<i64>) -> StudySummaryView {
    let summary = review_summary(&run.results);
    let mistake_cards = summary
        .mistakes
        .iter()
        .filter_map(|id| run.queue.iter().find(|card| &card.id == id))
        .map(|card| card_view(card, true))
        .collect();
    let score = run.mode.is_timed().then(|| blitz_score(&run.results));

    StudySummaryView {
        deck_id: run.deck_id.clone(),
        deck_name: run.deck_name.clone(),
        mode: run.mode.as_str().to_string(),
        summary,
        mistake_cards,
        points: score.map(|score| score.points),
        best_streak: score.map(|score| score.best_streak),
        record,
        record_beaten: run.record_beaten,
    }
}
