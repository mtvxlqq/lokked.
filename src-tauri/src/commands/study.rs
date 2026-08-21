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
//! the grade. Neither is accumulated by ticking.

use std::sync::Mutex;

use chrono::{DateTime, Local, Utc};
use serde::Serialize;
use tauri::State;

use crate::core::clock::Clock;
use crate::core::dayline::day_key;
use crate::core::review::Grade;
use crate::core::scheduler::shuffle;
use crate::core::stats::{review_summary, ReviewOutcome, ReviewSummary};
use crate::db::decks::DeckRepo;
use crate::db::reviews::{NewReview, ReviewRepo};
use crate::db::Database;
use crate::platform::clock::SystemClock;

use super::cards::CardView;
use super::settings::day_start;
use super::{CommandError, ErrorKind};

/// The only mode there is so far; blitz, marathon and «слабые» join it in M11.
const CLASSIC: &str = "classic";

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
    pub mode: String,
    pub total: usize,
    /// 1-based number of the card on screen; equals `total` on the last one.
    pub position: usize,
    pub answered: usize,
    pub revealed: bool,
    /// `null` once the run is over — the screen switches to the summary.
    pub card: Option<StudyCardView>,
    pub finished: bool,
}

/// The summary screen: the numbers, plus the cards that were missed.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct StudySummaryView {
    pub deck_id: String,
    pub deck_name: String,
    #[serde(flatten)]
    pub summary: ReviewSummary,
    /// The missed cards themselves, so the screen can list them by name.
    pub mistake_cards: Vec<StudyCardView>,
}

/// The run in progress, or the one that has just finished.
pub struct StudyRun {
    deck_id: String,
    deck_name: String,
    queue: Vec<CardView>,
    /// Index of the card on screen; equals `queue.len()` when the run is over.
    position: usize,
    shown_at: DateTime<Utc>,
    revealed_at: Option<DateTime<Utc>>,
    results: Vec<ReviewOutcome>,
}

/// Managed state: the run, or nothing.
#[derive(Default)]
pub struct StudyState(Mutex<Option<StudyRun>>);

impl StudyState {
    fn lock(&self) -> std::sync::MutexGuard<'_, Option<StudyRun>> {
        self.0.lock().expect("study mutex poisoned")
    }
}

fn no_run() -> CommandError {
    CommandError {
        kind: ErrorKind::Conflict,
        message: "прогон не идёт".to_string(),
    }
}

fn conflict(message: &str) -> CommandError {
    CommandError {
        kind: ErrorKind::Conflict,
        message: message.to_string(),
    }
}

fn card_view(card: &CardView, revealed: bool) -> StudyCardView {
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

fn view(run: &StudyRun) -> StudyView {
    let finished = run.position >= run.queue.len();
    let revealed = run.revealed_at.is_some();

    StudyView {
        deck_id: run.deck_id.clone(),
        deck_name: run.deck_name.clone(),
        mode: CLASSIC.to_string(),
        total: run.queue.len(),
        position: (run.position + 1).min(run.queue.len()),
        answered: run.results.len(),
        revealed,
        card: run
            .queue
            .get(run.position)
            .map(|card| card_view(card, revealed)),
        finished,
    }
}

/// Builds a run out of `cards`, shuffled with `seed`.
fn begin(
    deck_id: &str,
    deck_name: &str,
    mut cards: Vec<CardView>,
    seed: u64,
    clock: &dyn Clock,
) -> Result<StudyRun, CommandError> {
    if cards.is_empty() {
        return Err(conflict("в колоде нет карточек"));
    }

    shuffle(&mut cards, seed);

    Ok(StudyRun {
        deck_id: deck_id.to_string(),
        deck_name: deck_name.to_string(),
        queue: cards,
        position: 0,
        shown_at: clock.now(),
        revealed_at: None,
        results: Vec::new(),
    })
}

/// Starts a run through a deck, in a random order.
///
/// `seed` is a parameter so a run can be replayed exactly in a test; in the
/// app it comes from the clock.
pub fn start(
    db: &Database,
    state: &StudyState,
    clock: &dyn Clock,
    deck_id: &str,
    seed: u64,
) -> Result<StudyView, CommandError> {
    let deck = DeckRepo::new(db)
        .get(deck_id)?
        .filter(|deck| deck.deleted_at.is_none())
        .ok_or_else(|| CommandError::not_found("колода"))?;
    let cards = super::cards::list(db, deck_id)?;

    let run = begin(deck_id, &deck.name, cards, seed, clock)?;
    let drawn = view(&run);
    *state.lock() = Some(run);

    Ok(drawn)
}

/// The run as it stands, or `None` if there is none.
pub fn current(state: &StudyState) -> Option<StudyView> {
    state.lock().as_ref().map(view)
}

/// Turns the card over, remembering when.
pub fn reveal(state: &StudyState, clock: &dyn Clock) -> Result<StudyView, CommandError> {
    let mut active = state.lock();
    let run = active.as_mut().ok_or_else(no_run)?;

    if run.position >= run.queue.len() {
        return Err(conflict("прогон закончен"));
    }
    // Повторное раскрытие не сдвигает отсчёт: студент мог нажать дважды.
    if run.revealed_at.is_none() {
        run.revealed_at = Some(clock.now());
    }

    Ok(view(run))
}

/// Grades the card on screen, writes the answer down and moves on.
pub fn answer(
    db: &Database,
    state: &StudyState,
    clock: &dyn Clock,
    grade: Grade,
) -> Result<StudyView, CommandError> {
    let mut active = state.lock();
    let run = active.as_mut().ok_or_else(no_run)?;

    let Some(card) = run.queue.get(run.position).cloned() else {
        return Err(conflict("прогон закончен"));
    };
    let Some(revealed_at) = run.revealed_at else {
        return Err(conflict("сначала надо посмотреть ответ"));
    };

    let now = clock.now();
    let think_ms = (revealed_at - run.shown_at).num_milliseconds().max(0);
    let total_ms = (now - run.shown_at).num_milliseconds().max(0);

    // Пишется до того, как прогон сдвинется: если запись не удалась, карточка
    // остаётся на экране и ответ можно повторить.
    ReviewRepo::new(db).create(NewReview {
        card_id: &card.id,
        reviewed_at: now,
        day_key: &day_key(now, &Local, day_start(db)?),
        result: grade.as_str(),
        correct: grade.is_correct(),
        mode: CLASSIC,
        think_ms: Some(think_ms),
        total_ms: Some(total_ms),
        device_id: None,
    })?;

    run.results.push(ReviewOutcome {
        card_id: card.id,
        grade,
        total_ms,
    });
    run.position += 1;
    run.shown_at = now;
    run.revealed_at = None;

    Ok(view(run))
}

/// The numbers under a run. Available while it is still going, too — the
/// screen after the last card and the one for a run abandoned halfway are
/// the same screen.
pub fn summary(state: &StudyState) -> Result<StudySummaryView, CommandError> {
    let active = state.lock();
    let run = active.as_ref().ok_or_else(no_run)?;

    let summary = review_summary(&run.results);
    let mistake_cards = summary
        .mistakes
        .iter()
        .filter_map(|id| run.queue.iter().find(|card| &card.id == id))
        .map(|card| card_view(card, true))
        .collect();

    Ok(StudySummaryView {
        deck_id: run.deck_id.clone(),
        deck_name: run.deck_name.clone(),
        summary,
        mistake_cards,
    })
}

/// Starts a new run over just the cards missed in the one that finished.
pub fn repeat_mistakes(
    state: &StudyState,
    clock: &dyn Clock,
    seed: u64,
) -> Result<StudyView, CommandError> {
    let mut active = state.lock();
    let run = active.as_ref().ok_or_else(no_run)?;

    let missed = review_summary(&run.results).mistakes;
    let cards: Vec<CardView> = missed
        .iter()
        .filter_map(|id| run.queue.iter().find(|card| &card.id == id))
        .cloned()
        .collect();

    if cards.is_empty() {
        return Err(conflict("ошибок в этом прогоне не было"));
    }

    let next = begin(&run.deck_id, &run.deck_name, cards, seed, clock)?;
    let drawn = view(&next);
    *active = Some(next);

    Ok(drawn)
}

/// Ends the run. Answers already given stay in `reviews` — they happened.
pub fn stop(state: &StudyState) {
    *state.lock() = None;
}

/// A seed for a run that nobody asked to reproduce.
fn seed_from(clock: &dyn Clock) -> u64 {
    clock.now().timestamp_nanos_opt().unwrap_or(0) as u64
}

#[tauri::command]
pub fn study_start(
    db: State<'_, Database>,
    state: State<'_, StudyState>,
    deck_id: String,
) -> Result<StudyView, CommandError> {
    start(&db, &state, &SystemClock, &deck_id, seed_from(&SystemClock))
}

#[tauri::command]
pub fn study_current(state: State<'_, StudyState>) -> Option<StudyView> {
    current(&state)
}

#[tauri::command]
pub fn study_reveal(state: State<'_, StudyState>) -> Result<StudyView, CommandError> {
    reveal(&state, &SystemClock)
}

#[tauri::command]
pub fn study_answer(
    db: State<'_, Database>,
    state: State<'_, StudyState>,
    grade: String,
) -> Result<StudyView, CommandError> {
    answer(&db, &state, &SystemClock, Grade::parse(&grade)?)
}

#[tauri::command]
pub fn study_summary(state: State<'_, StudyState>) -> Result<StudySummaryView, CommandError> {
    summary(&state)
}

#[tauri::command]
pub fn study_repeat_mistakes(state: State<'_, StudyState>) -> Result<StudyView, CommandError> {
    repeat_mistakes(&state, &SystemClock, seed_from(&SystemClock))
}

#[tauri::command]
pub fn study_stop(state: State<'_, StudyState>) {
    stop(&state)
}
