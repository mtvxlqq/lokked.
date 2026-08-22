//! A duel: a blitz two to four people take turns at on one device.
//!
//! The state lives here, in Tauri's managed state, for the same reason a run
//! of cards does: whose turn it is, what has been answered and what that
//! scored are decisions, not decoration.
//!
//! Two rules shape everything in this module. Everyone answers **the same
//! cards in the same order** — the sequence is dealt once, at the start, and
//! replayed for each player — and **nobody sees anyone else's score** until
//! the last turn is over, which is why [`DuelView`] carries the current
//! player's points and nothing else.
//!
//! [`actions`] holds what can be done to a duel; [`api`] is the thin Tauri
//! surface over it.

use std::sync::Mutex;

use chrono::{DateTime, TimeDelta, Utc};
use serde::Serialize;

use crate::core::duel::{breakdown, winners, DuelSetup};
use crate::core::review::Grade;
use crate::core::stats::{blitz_score, ReviewOutcome};

use super::cards::CardView;
use super::study::{card_view, StudyCardView};
use super::{CommandError, ErrorKind};

pub mod actions;
pub mod api;

/// One player, as the duel screen shows them.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DuelPlayerView {
    pub name: String,
    pub is_owner: bool,
    /// Whether this player's turn is already over. Their score is not here:
    /// it stays hidden until the duel ends.
    pub played: bool,
}

/// What the duel screen draws.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DuelView {
    pub duel_id: String,
    pub deck_id: String,
    pub deck_name: String,
    pub players: Vec<DuelPlayerView>,
    /// Whose turn it is, 0-based, and their name.
    pub current_player: usize,
    pub current_name: String,
    /// 1-based turn number, and how many turns there are in all.
    pub turn: usize,
    pub turns: usize,
    /// Cards in one turn, and where this turn is in them.
    pub total: usize,
    pub position: usize,
    pub answered: usize,
    pub revealed: bool,
    /// `null` on the hand-over screen and once the duel is over.
    pub card: Option<StudyCardView>,
    pub deadline: Option<DateTime<Utc>>,
    pub seconds_per_card: i64,
    /// The current player's own score, so far, in their own turn.
    pub points: i64,
    pub streak: u32,
    /// Waiting for the next player to say they are ready.
    pub handover: bool,
    pub finished: bool,
}

/// One player's result, once everything may be shown.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DuelResultView {
    pub name: String,
    pub is_owner: bool,
    pub points: i64,
    pub correct: usize,
    pub answered: usize,
    pub best_streak: u32,
    pub winner: bool,
}

/// One card of the duel, and what each player said to it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DuelCardView {
    pub card_id: String,
    pub front: String,
    pub back: String,
    /// One entry per player, in turn order; `null` where a player never got
    /// that far.
    pub answers: Vec<Option<String>>,
}

/// The screen after the last turn.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DuelSummaryView {
    pub duel_id: String,
    pub deck_id: String,
    pub deck_name: String,
    pub cards: usize,
    pub seconds_per_card: i64,
    pub players: Vec<DuelResultView>,
    pub breakdown: Vec<DuelCardView>,
}

/// One player inside a running duel.
pub(crate) struct RunPlayer {
    pub(crate) name: String,
    pub(crate) is_owner: bool,
    /// The row in `duel_players` their answers are filed under.
    pub(crate) row_id: String,
    pub(crate) results: Vec<ReviewOutcome>,
}

impl RunPlayer {
    /// Cards recalled in a row at this moment.
    fn streak(&self) -> u32 {
        self.results
            .iter()
            .rev()
            .take_while(|result| result.grade.is_correct())
            .count() as u32
    }

    /// The longest such run of the whole turn.
    fn best_streak(&self) -> u32 {
        blitz_score(&self.results).best_streak
    }
}

/// The duel in progress, or the one that has just finished.
pub struct DuelRun {
    pub(crate) duel_id: String,
    pub(crate) deck_id: String,
    pub(crate) deck_name: String,
    pub(crate) day_key: String,
    pub(crate) players: Vec<RunPlayer>,
    /// The cards every player answers, in the order the reel spins them out.
    pub(crate) queue: Vec<CardView>,
    pub(crate) seconds_per_card: i64,
    /// Whose turn it is.
    pub(crate) current: usize,
    /// Which card of the turn is on screen.
    pub(crate) position: usize,
    pub(crate) shown_at: DateTime<Utc>,
    pub(crate) revealed_at: Option<DateTime<Utc>>,
    /// The device is being handed over: the turn has not started yet.
    pub(crate) handover: bool,
    pub(crate) finished: bool,
}

impl DuelRun {
    /// When the card on screen runs out of time.
    pub(crate) fn deadline(&self) -> Option<DateTime<Utc>> {
        (!self.handover && !self.finished)
            .then(|| self.shown_at + TimeDelta::seconds(self.seconds_per_card))
    }

    pub(crate) fn is_late(&self, now: DateTime<Utc>) -> bool {
        self.deadline().is_some_and(|deadline| now > deadline)
    }

    pub(crate) fn player(&self) -> &RunPlayer {
        &self.players[self.current]
    }

    pub(crate) fn player_mut(&mut self) -> &mut RunPlayer {
        let current = self.current;
        &mut self.players[current]
    }
}

/// Managed state: the duel, or nothing.
#[derive(Default)]
pub struct DuelState(Mutex<Option<DuelRun>>);

impl DuelState {
    pub(crate) fn lock(&self) -> std::sync::MutexGuard<'_, Option<DuelRun>> {
        self.0.lock().expect("duel mutex poisoned")
    }
}

pub(crate) fn no_duel() -> CommandError {
    CommandError {
        kind: ErrorKind::Conflict,
        message: "дуэль не идёт".to_string(),
    }
}

pub(crate) fn conflict(message: &str) -> CommandError {
    CommandError {
        kind: ErrorKind::Conflict,
        message: message.to_string(),
    }
}

/// A duel that has been written down and dealt, on its way into state.
pub(crate) struct Dealt<'a> {
    pub(crate) duel_id: String,
    pub(crate) deck_id: &'a str,
    pub(crate) deck_name: &'a str,
    pub(crate) day_key: &'a str,
    pub(crate) setup: &'a DuelSetup,
    pub(crate) players: Vec<RunPlayer>,
    /// The cards every player answers, already shuffled and cut to size.
    pub(crate) queue: Vec<CardView>,
    pub(crate) now: DateTime<Utc>,
}

/// Builds a duel out of a validated setup and the cards it will run over.
pub(crate) fn begin(dealt: Dealt<'_>) -> DuelRun {
    let Dealt {
        duel_id,
        deck_id,
        deck_name,
        day_key,
        setup,
        players,
        queue,
        now,
    } = dealt;

    DuelRun {
        duel_id,
        deck_id: deck_id.to_string(),
        deck_name: deck_name.to_string(),
        day_key: day_key.to_string(),
        players,
        queue,
        seconds_per_card: setup.seconds_per_card,
        current: 0,
        position: 0,
        shown_at: now,
        revealed_at: None,
        // Даже первый игрок начинает с «я готов»: дуэль стартует, когда
        // устройство у того, чей ход, а не когда закрыт экран настройки.
        handover: true,
        finished: false,
    }
}

pub(crate) fn view(run: &DuelRun) -> DuelView {
    let revealed = run.revealed_at.is_some();
    let showing_card = !run.handover && !run.finished;

    DuelView {
        duel_id: run.duel_id.clone(),
        deck_id: run.deck_id.clone(),
        deck_name: run.deck_name.clone(),
        players: run
            .players
            .iter()
            .enumerate()
            .map(|(index, player)| DuelPlayerView {
                name: player.name.clone(),
                is_owner: player.is_owner,
                played: run.finished || index < run.current,
            })
            .collect(),
        current_player: run.current,
        current_name: run.player().name.clone(),
        turn: run.current + 1,
        turns: run.players.len(),
        total: run.queue.len(),
        position: (run.position + 1).min(run.queue.len()),
        answered: run.player().results.len(),
        revealed,
        card: showing_card
            .then(|| run.queue.get(run.position))
            .flatten()
            .map(|card| card_view(card, revealed)),
        deadline: run.deadline(),
        seconds_per_card: run.seconds_per_card,
        points: blitz_score(&run.player().results).points,
        streak: run.player().streak(),
        handover: run.handover,
        finished: run.finished,
    }
}

/// The table after the last turn: scores, winners and the card-by-card
/// breakdown.
pub(crate) fn summarise(run: &DuelRun) -> DuelSummaryView {
    let points: Vec<i64> = run
        .players
        .iter()
        .map(|player| blitz_score(&player.results).points)
        .collect();
    let won = winners(&points);

    let answers: Vec<(usize, usize, Grade)> = run
        .players
        .iter()
        .enumerate()
        .flat_map(|(index, player)| {
            player
                .results
                .iter()
                .enumerate()
                .map(move |(position, result)| (index, position, result.grade))
        })
        .collect();
    let card_ids: Vec<String> = run.queue.iter().map(|card| card.id.clone()).collect();

    let breakdown = breakdown(&card_ids, run.players.len(), &answers)
        .into_iter()
        .zip(run.queue.iter())
        .map(|(row, card)| DuelCardView {
            card_id: row.card_id,
            front: card.front.clone(),
            back: card.back.clone(),
            answers: row
                .answers
                .iter()
                .map(|grade| grade.map(|grade| grade.as_str().to_string()))
                .collect(),
        })
        .collect();

    DuelSummaryView {
        duel_id: run.duel_id.clone(),
        deck_id: run.deck_id.clone(),
        deck_name: run.deck_name.clone(),
        cards: run.queue.len(),
        seconds_per_card: run.seconds_per_card,
        players: run
            .players
            .iter()
            .enumerate()
            .map(|(index, player)| DuelResultView {
                name: player.name.clone(),
                is_owner: player.is_owner,
                points: points[index],
                correct: player
                    .results
                    .iter()
                    .filter(|result| result.grade.is_correct())
                    .count(),
                answered: player.results.len(),
                best_streak: player.best_streak(),
                winner: won.contains(&index),
            })
            .collect(),
        breakdown,
    }
}
