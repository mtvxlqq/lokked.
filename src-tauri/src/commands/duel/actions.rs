//! What can be done to a duel: pick a deck, deal it, hand the device over,
//! answer, and read the table at the end.

use chrono::Local;

use crate::core::clock::Clock;
use crate::core::dayline::day_key;
use crate::core::duel::DuelSetup;
use crate::core::review::Grade;
use crate::core::scheduler::{shuffle, Rng};
use crate::core::stats::{blitz_score, ReviewOutcome};
use crate::db::decks::DeckRepo;
use crate::db::duels::{DuelRepo, NewDuel, NewDuelAnswer};
use crate::db::reviews::{NewReview, ReviewRepo};
use crate::db::Database;

use super::super::cards::CardView;
use super::super::decks::DeckView;
use super::super::settings::day_start;
use super::super::CommandError;
use super::{
    begin, conflict, no_duel, summarise, view, DuelState, DuelSummaryView, DuelView, RunPlayer,
};

/// The mode a duel's answers are written to `reviews` under.
///
/// Only the owner's answers are written at all, and they are marked as their
/// own thing: a duel is studying, but it is studying against a clock and an
/// audience, and the statistics screen should be able to tell.
const DUEL_MODE: &str = "duel";

/// Picks a deck at random — what the reel lands on when nobody chose.
///
/// The choice is made here rather than in the spinner for the same reason it
/// is in every other mode: what falls out is the backend's business, and the
/// reel is how it is shown.
pub fn pick_deck(db: &Database, seed: u64) -> Result<DeckView, CommandError> {
    let decks = super::super::decks::list(db)?;
    if decks.is_empty() {
        return Err(conflict("нет ни одной колоды"));
    }

    let index = Rng::new(seed).below(decks.len() as u64) as usize;

    Ok(decks[index].clone())
}

/// What a duel is started with.
///
/// A struct rather than six arguments: the setup screen sends all of it at
/// once, and `seed` is only separate from the rest so a test can replay a
/// duel exactly.
pub struct DuelStart<'a> {
    pub deck_id: &'a str,
    pub names: Vec<String>,
    pub cards: usize,
    pub seconds_per_card: i64,
    pub seed: u64,
}

/// Starts a duel over `deck_id`.
///
/// The sequence of cards is dealt once and answered by everyone in the same
/// order — that is what makes the scores comparable at the end.
pub fn start(
    db: &Database,
    state: &DuelState,
    clock: &dyn Clock,
    started: DuelStart<'_>,
) -> Result<DuelView, CommandError> {
    let DuelStart {
        deck_id,
        names,
        cards,
        seconds_per_card,
        seed,
    } = started;
    let setup = DuelSetup::new(names, cards, seconds_per_card)?;

    let deck = DeckRepo::new(db)
        .get(deck_id)?
        .filter(|deck| deck.deleted_at.is_none())
        .ok_or_else(|| CommandError::not_found("колода"))?;

    let mut pool: Vec<CardView> = super::super::cards::list(db, deck_id)?;
    if pool.len() < setup.cards {
        return Err(crate::core::duel::DuelError::NotEnoughCards {
            asked: setup.cards,
            has: pool.len(),
        }
        .into());
    }

    shuffle(&mut pool, seed);
    pool.truncate(setup.cards);

    let now = clock.now();
    let today = day_key(now, &Local, day_start(db)?);
    let repo = DuelRepo::new(db);
    let duel = repo.create(NewDuel {
        deck_id,
        day_key: &today,
        started_at: now,
        cards: setup.cards as i64,
        seconds_per_card: setup.seconds_per_card,
    })?;

    let mut players = Vec::with_capacity(setup.players.len());
    for (position, player) in setup.players.iter().enumerate() {
        let row_id = repo.add_player(&duel.id, &player.name, position as i64, player.is_owner)?;
        players.push(RunPlayer {
            name: player.name.clone(),
            is_owner: player.is_owner,
            row_id,
            results: Vec::new(),
        });
    }

    let run = begin(super::Dealt {
        duel_id: duel.id,
        deck_id,
        deck_name: &deck.name,
        day_key: &today,
        setup: &setup,
        players,
        queue: pool,
        now,
    });
    let drawn = view(&run);
    *state.lock() = Some(run);

    Ok(drawn)
}

/// The duel as it stands, or `None` if there is none.
pub fn current(state: &DuelState) -> Option<DuelView> {
    state.lock().as_ref().map(view)
}

/// The next player says they have the device. Their turn starts now.
pub fn begin_turn(state: &DuelState, clock: &dyn Clock) -> Result<DuelView, CommandError> {
    let mut active = state.lock();
    let run = active.as_mut().ok_or_else(no_duel)?;

    if run.finished {
        return Err(conflict("дуэль закончена"));
    }
    if !run.handover {
        return Err(conflict("ход уже идёт"));
    }

    run.handover = false;
    run.position = 0;
    run.shown_at = clock.now();
    run.revealed_at = None;

    Ok(view(run))
}

/// The reel has stopped: the card's time starts now.
///
/// Without this the spin would eat the first second and a half of a card
/// whose whole point is that it is timed. The clock is armed when the card
/// is actually readable, not when it was dealt.
pub fn settled(state: &DuelState, clock: &dyn Clock) -> Result<DuelView, CommandError> {
    let mut active = state.lock();
    let run = active.as_mut().ok_or_else(no_duel)?;

    if run.handover || run.finished {
        return Err(conflict("сейчас не ход"));
    }
    // Повторное сообщение о том, что барабан встал, время не продлевает.
    if run.revealed_at.is_none() {
        run.shown_at = clock.now();
    }

    Ok(view(run))
}

/// Turns the card over, remembering when.
pub fn reveal(state: &DuelState, clock: &dyn Clock) -> Result<DuelView, CommandError> {
    let mut active = state.lock();
    let run = active.as_mut().ok_or_else(no_duel)?;

    if run.handover || run.finished {
        return Err(conflict("сейчас не ход"));
    }
    if run.revealed_at.is_none() {
        run.revealed_at = Some(clock.now());
    }

    Ok(view(run))
}

/// Grades the card on screen, writes the answer down and moves on.
///
/// A card that ran out of time counts as «не помню» whatever was pressed —
/// same rule as the blitz this is built on.
pub fn answer(
    db: &Database,
    state: &DuelState,
    clock: &dyn Clock,
    grade: Grade,
) -> Result<DuelView, CommandError> {
    let mut active = state.lock();
    let run = active.as_mut().ok_or_else(no_duel)?;

    if run.handover || run.finished {
        return Err(conflict("сейчас не ход"));
    }

    let Some(card) = run.queue.get(run.position).cloned() else {
        return Err(conflict("ход закончен"));
    };

    let now = clock.now();
    let late = run.is_late(now);
    if !late && run.revealed_at.is_none() {
        return Err(conflict("сначала надо посмотреть ответ"));
    }

    let grade = if late { Grade::Again } else { grade };
    let total_ms = (now - run.shown_at).num_milliseconds().max(0);
    let position = run.position as i64;
    let think_ms = run
        .revealed_at
        .map(|revealed| (revealed - run.shown_at).num_milliseconds().max(0));

    // Пишется до того, как ход сдвинется: если запись не удалась, карточка
    // остаётся на экране и ответ можно повторить.
    let repo = DuelRepo::new(db);
    repo.record_answer(NewDuelAnswer {
        duel_id: &run.duel_id,
        player_id: &run.player().row_id,
        card_id: &card.id,
        position,
        result: grade.as_str(),
        correct: grade.is_correct(),
        total_ms: Some(total_ms),
    })?;

    // Только ответы владельца устройства попадают в его собственную историю:
    // вечер в гостях — не чужая статистика и не чужие веса карточек.
    if run.player().is_owner {
        ReviewRepo::new(db).create(NewReview {
            card_id: &card.id,
            reviewed_at: now,
            day_key: &run.day_key,
            result: grade.as_str(),
            correct: grade.is_correct(),
            mode: DUEL_MODE,
            think_ms,
            total_ms: Some(total_ms),
            device_id: None,
        })?;
    }

    run.player_mut().results.push(ReviewOutcome {
        card_id: card.id,
        grade,
        total_ms,
    });
    run.position += 1;
    run.shown_at = now;
    run.revealed_at = None;

    if run.position >= run.queue.len() {
        end_turn(db, run, clock)?;
    }

    Ok(view(run))
}

/// Closes a turn: writes down what it scored and passes the device on.
fn end_turn(
    db: &Database,
    run: &mut super::DuelRun,
    clock: &dyn Clock,
) -> Result<(), CommandError> {
    let score = blitz_score(&run.player().results);
    let correct = run
        .player()
        .results
        .iter()
        .filter(|result| result.grade.is_correct())
        .count() as i64;

    let repo = DuelRepo::new(db);
    repo.save_score(
        &run.player().row_id,
        score.points,
        correct,
        score.best_streak as i64,
    )?;

    if run.current + 1 < run.players.len() {
        run.current += 1;
        run.position = 0;
        run.handover = true;
        run.revealed_at = None;
    } else {
        run.finished = true;
        repo.finish(&run.duel_id, clock.now())?;
    }

    Ok(())
}

/// The table at the end. Available on a duel left halfway too — the screen
/// after the last card and the one for a duel abandoned mid-turn are the
/// same screen.
pub fn summary(state: &DuelState) -> Result<DuelSummaryView, CommandError> {
    let active = state.lock();
    let run = active.as_ref().ok_or_else(no_duel)?;

    Ok(summarise(run))
}

/// Ends the duel. Answers already given stay in `duel_answers` — they
/// happened.
pub fn stop(state: &DuelState) {
    *state.lock() = None;
}
