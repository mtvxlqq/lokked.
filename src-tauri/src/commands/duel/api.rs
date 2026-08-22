//! The Tauri surface of a duel.
//!
//! Nothing but argument shuffling: every one of these is a line or two on top
//! of [`super::actions`], which is where the duel actually happens and where
//! the tests get at it without a running Tauri app.

use tauri::State;

use crate::core::clock::Clock;
use crate::core::duel::DEFAULT_DUEL_CARDS;
use crate::core::review::Grade;
use crate::db::Database;
use crate::platform::clock::SystemClock;

use super::super::decks::DeckView;
use super::super::CommandError;
use super::actions::{self, DuelStart};
use super::{DuelState, DuelSummaryView, DuelView};

/// A seed for a duel nobody asked to reproduce.
fn seed_from(clock: &dyn Clock) -> u64 {
    clock.now().timestamp_nanos_opt().unwrap_or(0) as u64
}

#[tauri::command]
pub fn duel_pick_deck(db: State<'_, Database>) -> Result<DeckView, CommandError> {
    actions::pick_deck(&db, seed_from(&SystemClock))
}

#[tauri::command]
pub fn duel_start(
    db: State<'_, Database>,
    state: State<'_, DuelState>,
    deck_id: String,
    players: Vec<String>,
    cards: Option<usize>,
    seconds_per_card: i64,
) -> Result<DuelView, CommandError> {
    actions::start(
        &db,
        &state,
        &SystemClock,
        DuelStart {
            deck_id: &deck_id,
            names: players,
            cards: cards.unwrap_or(DEFAULT_DUEL_CARDS),
            seconds_per_card,
            seed: seed_from(&SystemClock),
        },
    )
}

#[tauri::command]
pub fn duel_current(state: State<'_, DuelState>) -> Option<DuelView> {
    actions::current(&state)
}

#[tauri::command]
pub fn duel_begin_turn(state: State<'_, DuelState>) -> Result<DuelView, CommandError> {
    actions::begin_turn(&state, &SystemClock)
}

#[tauri::command]
pub fn duel_settled(state: State<'_, DuelState>) -> Result<DuelView, CommandError> {
    actions::settled(&state, &SystemClock)
}

#[tauri::command]
pub fn duel_reveal(state: State<'_, DuelState>) -> Result<DuelView, CommandError> {
    actions::reveal(&state, &SystemClock)
}

#[tauri::command]
pub fn duel_answer(
    db: State<'_, Database>,
    state: State<'_, DuelState>,
    grade: String,
) -> Result<DuelView, CommandError> {
    actions::answer(&db, &state, &SystemClock, Grade::parse(&grade)?)
}

/// The clock ran out. Same as answering «не помню».
#[tauri::command]
pub fn duel_timeout(
    db: State<'_, Database>,
    state: State<'_, DuelState>,
) -> Result<DuelView, CommandError> {
    actions::answer(&db, &state, &SystemClock, Grade::Again)
}

#[tauri::command]
pub fn duel_summary(state: State<'_, DuelState>) -> Result<DuelSummaryView, CommandError> {
    actions::summary(&state)
}

#[tauri::command]
pub fn duel_stop(state: State<'_, DuelState>) {
    actions::stop(&state)
}
